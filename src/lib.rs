#![warn(missing_docs)]
#![cfg_attr(doc, feature(doc_auto_cfg))]

//! A Rust library for estimating quantiles in a stream,
//! using [ClickHouse t-digest][ClickHouseRefTDigest] data structure.
//!
//! The [t-digest][Dunning19] data structure is designed around computing
//! accurate quantile estimates from streaming data. Two t-digests can be
//! merged, making the data structure well suited for map-reduce settings.
//!
//! [Repository]
//!
//! [ClickHouseRefTDigest]: https://clickhouse.com/docs/en/sql-reference/aggregate-functions/reference/quantiletdigest/
//! [Dunning19]: https://github.com/tdunning/t-digest/blob/main/docs/t-digest-paper/histo.pdf
//! [Repository]: https://github.com/vivienm/rust-tdigest-ch
//!
//! # Examples
//!
//! ```
//! use tdigest_ch::TDigest;
//!
//! let mut digest = TDigest::new();
//!
//! // Add some elements.
//! digest.insert(1.0);
//! digest.insert(2.0);
//! digest.insert(3.0);
//!
//! // Get the median of the distribution.
//! let quantile = digest.quantile(0.5);
//! assert_eq!(quantile, 2.0);
//! ```

use std::{
    cmp::Ordering,
    ops::{BitOr, BitOrAssign},
};

/// Stores the weight of points around their mean value.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Centroid {
    mean: f32,
    count: usize,
}

#[cfg(feature = "serde")]
impl serde::Serialize for Centroid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (self.mean, self.count).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Centroid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (mean, count) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self { mean, count })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Config {
    epsilon: f32,
    max_centroids: usize,
    max_unmerged: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            epsilon: 0.01,
            max_centroids: 2048,
            max_unmerged: 2048,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (self.epsilon, self.max_centroids, self.max_unmerged).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (epsilon, max_centroids, max_unmerged) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self {
            epsilon,
            max_centroids,
            max_unmerged,
        })
    }
}

/// A `TDigestBuilder` can be used to create a `TDigest` with custom
/// configuration.
///
/// # Examples
///
/// ```
/// use tdigest_ch::TDigestBuilder;
///
/// let mut builder = TDigestBuilder::new();
/// builder.max_centroids(1024);
/// builder.max_unmerged(1024);
///
/// let digest = builder.build();
/// ```
#[derive(Debug)]
pub struct TDigestBuilder {
    config: Config,
}

impl TDigestBuilder {
    /// Constructs a new `TDigestBuilder`.
    ///
    /// This is the same as `TDigest::builder()`.
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Returns a `TDigest` that uses this `TDigestBuilder` configuration.
    pub fn build(self) -> TDigest {
        let centroids = Vec::with_capacity(self.config.max_centroids);
        TDigest {
            config: self.config,
            centroids,
            count: 0,
            unmerged: 0,
        }
    }

    pub fn epsilon(&mut self, epsilon: f32) -> &mut Self {
        self.config.epsilon = epsilon;
        self
    }

    pub fn max_centroids(&mut self, max_centroids: usize) -> &mut Self {
        self.config.max_centroids = max_centroids;
        self
    }

    pub fn max_unmerged(&mut self, max_unmerged: usize) -> &mut Self {
        self.config.max_unmerged = max_unmerged;
        self
    }
}

impl Default for TDigestBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn interpolate(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let k = (x - x1) / (x2 - x1);
    (1. - k) * y1 + k * y2
}

#[inline]
fn can_be_merged(l_mean: f64, r_mean: f32) -> bool {
    l_mean == r_mean as f64 || (!l_mean.is_infinite() && !r_mean.is_infinite())
}

fn cmp_f32(lhs: f32, rhs: f32) -> Ordering {
    match lhs.partial_cmp(&rhs) {
        Some(ordering) => ordering,
        None => {
            if lhs.is_nan() {
                if rhs.is_nan() {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        }
    }
}

/// T-digest data structure for approximating the quantiles of a distribution.
///
/// # Examples
///
/// ```
/// use tdigest_ch::TDigest;
///
/// let mut digest = TDigest::new();
///
/// // Add some elements.
/// digest.insert(1.0);
/// digest.insert(2.0);
/// digest.insert(3.0);
///
/// // Get the median of the distribution.
/// let quantile = digest.quantile(0.5);
/// assert_eq!(quantile, 2.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TDigest {
    config: Config,
    centroids: Vec<Centroid>,
    count: usize,
    unmerged: usize,
}

impl TDigest {
    /// Creates an empty `TDigest`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    /// let digest = TDigest::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Creates a `TDigestBuilder` to configure a `TDigest`.
    ///
    /// This is the same as `TDigestBuilder::new()`.
    #[inline]
    pub fn builder() -> TDigestBuilder {
        TDigestBuilder::new()
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut a = TDigest::from([-10.0, 1.0, 2.0, 2.0, 3.0]);
    /// let mut b = TDigest::from([-20.0, 5.0, 43.0]);
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 8);
    /// assert!(b.is_empty());
    /// ```
    pub fn append(&mut self, other: &mut TDigest) {
        self.bitor_assign(other);
        other.clear();
    }

    /// Returns the number of elements in the t-digest.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::new();
    /// assert_eq!(digest.len(), 0);
    /// digest.insert(1.0);
    /// assert_eq!(digest.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the t-digest contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::new();
    /// assert!(digest.is_empty());
    /// digest.insert(1.0);
    /// assert!(!digest.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the t-digest, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::new();
    /// digest.insert(1.0);
    /// digest.clear();
    /// assert!(digest.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.centroids.clear();
        self.count = 0;
        self.unmerged = 0;
    }

    /// Returns the estimated quantile of the t-digest.
    ///
    /// This method expects `self` to be mutable, since the t-digest may be
    /// compressed. If you require an immutable, shared reference to compute
    /// quantiles, consider using `quantiles` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::from([1.0, 2.0, 3.0, 4.0, 5.0]);
    /// assert_eq!(digest.quantile(0.0), 1.0);
    /// assert_eq!(digest.quantile(0.5), 3.0);
    /// assert_eq!(digest.quantile(1.0), 5.0);
    /// ```
    pub fn quantile(&mut self, level: f64) -> f32 {
        self.compress();
        self.quantile_uncompressed(level)
    }

    fn quantile_uncompressed(&self, level: f64) -> f32 {
        // Calculates the quantile q [0, 1] based on the digest.
        // For an empty digest returns NaN.
        if self.centroids.is_empty() {
            return f32::NAN;
        }

        if self.centroids.len() == 1 {
            return self.centroids[0].mean;
        }

        let x = level * self.count as f64;
        let mut prev_x = 0f64;
        let mut sum = 0usize;
        let mut prev = self.centroids[0];

        for c in self.centroids.iter() {
            let current_x = sum as f64 + c.count as f64 * 0.5;

            if current_x >= x {
                // Special handling of singletons.
                let mut left = prev_x;
                if prev.count == 1 {
                    left += 0.5;
                }
                let mut right = current_x;
                if c.count == 1 {
                    right -= 0.5;
                }

                return {
                    if x <= left {
                        prev.mean
                    } else if x >= right {
                        c.mean
                    } else {
                        interpolate(x as f32, left as f32, prev.mean, right as f32, c.mean)
                    }
                };
            }

            sum += c.count;
            prev = *c;
            prev_x = current_x;
        }

        self.centroids.last().unwrap().mean
    }

    /// Creates an immutable quantile estimator from the t-digest.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    ///
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::from([1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let quantiles = digest.quantiles();
    ///
    /// thread::scope(|s| {
    ///     s.spawn(|| {
    ///         assert_eq!(quantiles.get(0.0), 1.0);
    ///     });
    ///     s.spawn(|| {
    ///         assert_eq!(quantiles.get(0.5), 3.0);
    ///     });
    ///     s.spawn(|| {
    ///         assert_eq!(quantiles.get(1.0), 5.0);
    ///     });
    /// });
    /// ```
    pub fn quantiles(&mut self) -> Quantiles<'_> {
        self.compress();
        Quantiles { digest: self }
    }

    /// Adds a value to the t-digest.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::new();
    ///
    /// digest.insert(1.0);
    /// digest.insert(2.0);
    /// assert_eq!(digest.len(), 2);
    /// ```
    #[inline]
    pub fn insert(&mut self, value: f32) {
        self.insert_many(value, 1);
    }

    /// Adds multiple values to the t-digest.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::new();
    ///
    /// digest.insert_many(1.0, 1);
    /// digest.insert_many(2.0, 2);
    /// assert_eq!(digest.len(), 3);
    /// ```
    pub fn insert_many(&mut self, value: f32, count: usize) {
        if count == 0 || value.is_nan() {
            // Count 0 breaks compress() assumptions, NaN breaks sort(). We treat them as no
            // sample.
            return;
        }
        self.insert_centroid(&Centroid { mean: value, count });
    }

    fn insert_centroid(&mut self, centroid: &Centroid) {
        self.count += centroid.count;
        self.unmerged += 1;
        self.centroids.push(*centroid);
        if self.unmerged > self.config.max_unmerged {
            self.compress();
        }
    }

    fn compress(&mut self) {
        // Performs compression of accumulated centroids
        // When merging, the invariant is retained to the maximum size of each centroid
        // that does not exceed `4 q (1 - q) \ delta N`.
        if self.unmerged > 0 || self.centroids.len() > self.config.max_centroids {
            self.centroids.sort_by(|l, r| cmp_f32(l.mean, r.mean));

            let mut l_index = 0;

            // Compiler is unable to do this optimization.
            let count_epsilon_4 = self.count as f64 * self.config.epsilon as f64 * 4.;
            let mut sum = 0;
            let (mut l_mean, mut l_count) = {
                let l = self.centroids.first().unwrap();
                (l.mean as f64, l.count)
            };
            for r_index in 1..self.centroids.len() {
                let r = self.centroids[r_index];
                // N.B. We cannot merge all the same values into single centroids because this
                // will lead to unbalanced compression and wrong results.
                // For more information see: https://arxiv.org/abs/1902.04023.

                // The ratio of the part of the histogram to l, including the half l to the
                // entire histogram. That is, what level quantile in position l.
                let ql = (sum as f64 + l_count as f64 * 0.5) / self.count as f64;
                let mut err = ql * (1. - ql);

                // The ratio of the portion of the histogram to l, including l and half r to the
                // entire histogram. That is, what level is the quantile in position r.
                let qr = (sum as f64 + l_count as f64 + r.count as f64 * 0.5) / self.count as f64;
                let err2 = qr * (1. - qr);

                if err > err2 {
                    err = err2;
                }

                let k = count_epsilon_4 * err;

                // The ratio of the weight of the glued column pair to all values is not
                // greater, than epsilon multiply by a certain quadratic
                // coefficient, which in the median is 1 (4 * 1/2 * 1/2), and at
                // the edges decreases and is approximately equal to the
                // distance to the edge * 4.

                if l_count as f64 + r.count as f64 <= k && can_be_merged(l_mean, r.mean) {
                    // It is possible to merge left and right.
                    // The left column "eats" the right.
                    l_count += r.count;
                    if r.mean as f64 != l_mean {
                        // Handling infinities of the same sign well.
                        // Symmetric algo (M1*C1 + M2*C2)/(C1+C2) is numerically better, but slower.
                        l_mean += r.count as f64 * (r.mean as f64 - l_mean) / l_count as f64;
                    }
                    self.centroids[l_index] = Centroid {
                        mean: l_mean as f32,
                        count: l_count,
                    };
                } else {
                    // Not enough capacity, check the next pair.
                    // Not l_count, otherwise actual sum of elements will be different.
                    sum += self.centroids[l_index].count;
                    l_index += 1;

                    // We skip all the values "eaten" earlier.
                    while l_index != r_index {
                        self.centroids[l_index].count = 0;
                        l_index += 1;
                    }
                    (l_mean, l_count) = {
                        let l = self.centroids[l_index];
                        (l.mean as f64, l.count)
                    };
                }
            }
            // Update count, it might be different due to += inaccuracy
            self.count = sum + l_count;

            // At the end of the loop, all values to the right of l were "eaten".
            self.centroids.retain(|c| c.count != 0);
            self.unmerged = 0;
        }

        // Ensures centroids.size() < max_centroids, independent of unprovable floating
        // point blackbox above.
        self.compress_brute();
    }

    fn compress_brute(&mut self) {
        if self.centroids.len() <= self.config.max_centroids {
            return;
        }
        let batch_size =  // At least 2.
            (self.centroids.len() + self.config.max_centroids - 1) / self.config.max_centroids;
        debug_assert!(batch_size >= 2);

        let mut l_index = 0;
        let mut sum = 0;
        // We have high-precision temporaries for numeric stability
        let (mut l_mean, mut l_count) = {
            let l = self.centroids.first().unwrap();
            (l.mean as f64, l.count)
        };
        let mut batch_pos = 0usize;

        for r_index in 1..self.centroids.len() {
            let r = self.centroids[r_index];
            if batch_pos < batch_size - 1 {
                // The left column "eats" the right. Middle of the batch.
                l_count += r.count;
                if r.mean as f64 != l_mean {
                    // Handling infinities of the same sign well.
                    // Symmetric algo (M1*C1 + M2*C2)/(C1+C2) is numerically better, but slower.
                    l_mean += r.count as f64 * (r.mean as f64 - l_mean) / l_count as f64;
                }
                self.centroids[l_index] = Centroid {
                    mean: l_mean as f32,
                    count: l_count,
                };
                batch_pos += 1;
            } else {
                // End of the batch, start the next one.
                if !self.centroids[l_index].mean.is_nan() {
                    // Skip writing batch result if we compressed something to nan.
                    // Not l_count, otherwise actual sum of elements will be different.
                    sum += self.centroids[l_index].count;
                    l_index += 1;
                }

                while l_index != r_index {
                    // We skip all the values "eaten" earlier.
                    self.centroids[l_index].count = 0;
                    l_index += 1;
                }
                (l_mean, l_count) = {
                    let l = self.centroids[l_index];
                    (l.mean as f64, l.count)
                };
                batch_pos = 0;
            }
        }

        if !self.centroids[l_index].mean.is_nan() {
            // Update count, it might be different due to += inaccuracy.
            self.count = sum + l_count;
        } else {
            // Skip writing last batch if (super unlikely) it's nan.
            self.count = sum;
            self.centroids[l_index].count = 0;
        }
        self.centroids.retain(|c| c.count != 0);
        // Here centroids.len() <= params.max_centroids.
        debug_assert!(self.centroids.len() <= self.config.max_centroids);
    }
}

impl BitOr<&TDigest> for &TDigest {
    type Output = TDigest;

    /// Returns the union of `self` and `rhs` as a new `TDigest`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let a = TDigest::from([1.0, 2.0, 3.0]);
    /// let b = TDigest::from([3.0, 4.0, 5.0]);
    ///
    /// let mut c = &a | &b;
    ///
    /// assert_eq!(c.len(), 6);
    /// assert_eq!(c.quantile(0.5), 3.0);
    /// ```
    fn bitor(self, rhs: &TDigest) -> TDigest {
        let mut result = self.clone();
        result |= rhs;
        result
    }
}

impl BitOrAssign<&TDigest> for TDigest {
    /// Merges `self` and `rhs` into `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut a = TDigest::from([1.0, 2.0, 3.0]);
    /// let b = TDigest::from([3.0, 4.0, 5.0]);
    ///
    /// a |= &b;
    ///
    /// assert_eq!(a.len(), 6);
    /// assert_eq!(a.quantile(0.5), 3.0);
    /// ```
    fn bitor_assign(&mut self, rhs: &TDigest) {
        for c in &rhs.centroids {
            self.insert_centroid(c);
        }
    }
}

impl Default for TDigest {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<f32> for TDigest {
    fn extend<I: IntoIterator<Item = f32>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<const N: usize> From<[f32; N]> for TDigest {
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let digest1 = TDigest::from([1.0, 2.0, 3.0, 4.0]);
    /// let digest2: TDigest = [1.0, 2.0, 3.0, 4.0].into();
    /// assert_eq!(digest1, digest2);
    /// ```
    fn from(array: [f32; N]) -> Self {
        let mut digest = TDigest::new();
        for value in array.iter() {
            digest.insert(*value);
        }
        digest
    }
}

impl FromIterator<f32> for TDigest {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        let mut digest = TDigest::new();
        digest.extend(iter);
        digest
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&self.config, &self.centroids, self.count, self.unmerged).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (config, centroids, count, unmerged) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self {
            config,
            centroids,
            count,
            unmerged,
        })
    }
}

/// Estimates quantiles of a t-digest.
///
/// This `struct` is created by the [`quantiles`] method on [`TDigest`]. See its
/// documentation for more.
///
/// [`quantiles`]: TDigest::quantiles
pub struct Quantiles<'a> {
    digest: &'a TDigest,
}

impl<'a> Quantiles<'a> {
    /// Returns the estimated quantile of the t-digest.
    ///
    /// # Examples
    ///
    /// ```
    /// use tdigest_ch::TDigest;
    ///
    /// let mut digest = TDigest::from([1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let quantiles = digest.quantiles();
    /// assert_eq!(quantiles.get(0.0), 1.0);
    /// assert_eq!(quantiles.get(0.5), 3.0);
    /// assert_eq!(quantiles.get(1.0), 5.0);
    /// ```
    pub fn get(&self, level: f64) -> f32 {
        self.digest.quantile_uncompressed(level)
    }
}

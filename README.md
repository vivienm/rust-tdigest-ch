# tdigest-ch

A Rust library for estimating quantiles in a stream,
using [ClickHouse t-digest][ClickHouseRefTDigest] data structure.

The [t-digest][Dunning19] data structure is designed around computing
accurate quantile estimates from streaming data. Two t-digests can be merged,
making the data structure well suited for map-reduce settings.

[Documentation]

[ClickHouseRefTDigest]: https://clickhouse.com/docs/en/sql-reference/aggregate-functions/reference/quantiletdigest/
[Dunning19]: https://github.com/tdunning/t-digest/blob/main/docs/t-digest-paper/histo.pdf
[Documentation]: https://vivienm.github.io/rust-tdigest-ch/docs/tdigest_ch/

## Example

```rust
use tdigest_ch::TDigest;

let mut digest = TDigest::new();

// Add some elements.
digest.insert(1.0);
digest.insert(2.0);
digest.insert(3.0);

// Get the median of the distribution.
let quantile = digest.quantile(0.5);
assert_eq!(quantile, 2.0);
```

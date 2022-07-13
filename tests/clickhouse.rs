use ch_tdigest::TDigest;

#[test]
fn example() {
    let mut digest = TDigest::from([1.0, 2.0, 3.0]);
    assert_eq!(digest.quantile(0.5), 2.0);
}

#[test]
fn negative() {
    let mut digest = TDigest::from([-1.0, -2.0, -3.0]);
    assert_eq!(digest.quantile(0.5), -2.0);
}

#[test]
fn infinity_1() {
    let mut digest = TDigest::new();
    digest.insert(f32::INFINITY);
    for _ in 1..500_000 {
        digest.insert(f32::NEG_INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(f32::INFINITY);
    }
    assert_eq!(digest.quantile(0.01), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.1), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.25), f32::NEG_INFINITY);
    assert!(digest.quantile(0.5).is_nan());
    assert_eq!(digest.quantile(0.75), f32::INFINITY);
    assert_eq!(digest.quantile(0.9), f32::INFINITY);
    assert_eq!(digest.quantile(0.99), f32::INFINITY);
}

#[test]
fn infinity_1b() {
    let mut digest = TDigest::new();
    for _ in 0..500_000 {
        digest.insert(f32::INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(f32::NEG_INFINITY);
    }
    assert_eq!(digest.quantile(0.01), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.1), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.25), f32::NEG_INFINITY);
    assert!(digest.quantile(0.5).is_nan());
    assert_eq!(digest.quantile(0.75), f32::INFINITY);
    assert_eq!(digest.quantile(0.9), f32::INFINITY);
    assert_eq!(digest.quantile(0.99), f32::INFINITY);
}

#[test]
fn infinity_1c() {
    let mut digest = TDigest::new();
    for _ in 0..500_000 {
        digest.insert(f32::INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(0.);
    }
    assert_eq!(digest.quantile(0.01), 0.0);
    assert_eq!(digest.quantile(0.1), 0.0);
    assert_eq!(digest.quantile(0.25), 0.0);
    assert_eq!(digest.quantile(0.5), f32::INFINITY);
    assert_eq!(digest.quantile(0.75), f32::INFINITY);
    assert_eq!(digest.quantile(0.9), f32::INFINITY);
    assert_eq!(digest.quantile(0.99), f32::INFINITY);
}

#[test]
fn infinity_1d() {
    let mut digest = TDigest::new();
    digest.insert(f32::INFINITY);
    for _ in 1..500_000 {
        digest.insert(f32::NEG_INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(0.);
    }
    assert_eq!(digest.quantile(0.01), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.1), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.25), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.5), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.75), 0.0);
    assert_eq!(digest.quantile(0.9), 0.0);
    assert_eq!(digest.quantile(0.99), 0.0);
}

#[test]
fn infinity_1e() {
    let mut digest = TDigest::new();
    digest.insert(0.);
    for _ in 1..500_000 {
        digest.insert(f32::INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(f32::NEG_INFINITY);
    }
    assert_eq!(digest.quantile(0.01), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.1), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.25), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.5), 0.0);
    assert_eq!(digest.quantile(0.75), f32::INFINITY);
    assert_eq!(digest.quantile(0.9), f32::INFINITY);
    assert_eq!(digest.quantile(0.99), f32::INFINITY);
}

#[test]
fn infinity_1f() {
    let mut digest = TDigest::new();
    digest.insert(0.);
    for _ in 1..500_000 {
        digest.insert(f32::NEG_INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(f32::INFINITY);
    }
    assert_eq!(digest.quantile(0.01), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.1), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.25), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.5), 0.0);
    assert_eq!(digest.quantile(0.75), f32::INFINITY);
    assert_eq!(digest.quantile(0.9), f32::INFINITY);
    assert_eq!(digest.quantile(0.99), f32::INFINITY);
}

#[test]
fn infinity_2() {
    let mut values = Vec::with_capacity(300);
    for _ in 0..150 {
        values.push(f32::INFINITY);
        values.push(f32::NEG_INFINITY);
    }
    let mut digest = TDigest::from_iter(values.iter().copied());
    assert_eq!(digest.quantile(0.05), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.5), f32::NEG_INFINITY);
    assert_eq!(digest.quantile(0.95), f32::INFINITY);
}

#[test]
fn infinity_3() {
    let mut digest = TDigest::from([f32::INFINITY; 300]);
    assert_eq!(digest.quantile(0.5), f32::INFINITY);

    let mut digest = TDigest::from([f32::NEG_INFINITY; 300]);
    assert_eq!(digest.quantile(0.5), f32::NEG_INFINITY);
}

#[test]
fn infinity_4() {
    for array in [
        [f32::INFINITY, 0.0, f32::NEG_INFINITY],
        [f32::NEG_INFINITY, 0.0, f32::INFINITY],
        [f32::INFINITY, f32::NEG_INFINITY, 0.0],
        [f32::NEG_INFINITY, f32::INFINITY, 0.0],
    ] {
        assert_eq!(TDigest::from(array).quantile(0.5), 0.0);
    }
    for array in [
        [
            f32::INFINITY,
            f32::INFINITY,
            0.0,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            0.0,
        ],
        [
            f32::INFINITY,
            f32::INFINITY,
            0.0,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            0.0,
        ],
        [
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            0.0,
            f32::INFINITY,
            f32::INFINITY,
            -0.0,
        ],
    ] {
        assert_eq!(TDigest::from(array).quantile(0.5), 0.0);
    }
}

#[test]
fn infinity_6() {
    let mut digest = TDigest::from_iter((0..500).map(|i| i as f32));
    assert_eq!(digest.quantile(f64::NEG_INFINITY), 0.0);
    assert_eq!(digest.quantile(f64::INFINITY), 499.0);
}

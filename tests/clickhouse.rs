use ch_tdigest::TDigest;

#[test]
fn negative() {
    // Source: https://github.com/ClickHouse/ClickHouse/blob/5e34f48a181744a9f9241e3da0522eeaf9c68b84/tests/queries/0_stateless/00649_quantile_tdigest_negative.sql.
    let mut digest = TDigest::from([-1.0, -2.0, -3.0]);
    assert_eq!(digest.quantile(0.5), -2.0);
}

#[test]
fn infinity_1() {
    // Source: https://github.com/ClickHouse/ClickHouse/blob/5e34f48a181744a9f9241e3da0522eeaf9c68b84/tests/queries/0_stateless/02286_quantile_tdigest_infinity.sql.
    let mut digest = TDigest::new();
    digest.insert(f32::INFINITY);
    for _ in 1..500_000 {
        digest.insert(f32::NEG_INFINITY);
    }
    for _ in 500_000..1_000_000 {
        digest.insert(f32::INFINITY);
    }
    assert!(digest.quantile(0.5).is_nan());
}

#[test]
fn infinity_2() {
    // Source: https://github.com/ClickHouse/ClickHouse/blob/5e34f48a181744a9f9241e3da0522eeaf9c68b84/tests/queries/0_stateless/02286_quantile_tdigest_infinity.sql.
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

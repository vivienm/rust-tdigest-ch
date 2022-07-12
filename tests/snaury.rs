// Source: https://github.com/snaury/tdigest-go/blob/master/tdigest_test.go.

use ch_tdigest::TDigest;

#[test]
fn digest_10() {
    let mut digest = TDigest::new();
    digest.extend((1u8..=10).map(f32::from));
    assert_eq!(digest.len(), 10);

    for (quantile, expected) in [(0., 1.), (0.1, 1.), (0.5, 5.), (0.9, 9.), (1., 10.)] {
        assert_eq!(digest.quantile(quantile), expected);
    }
}

#[test]
#[ignore]
fn digest_1_000_000() {
    let mut digest = TDigest::new();
    digest.extend((1u32..=1_000_000).map(|value| value as f32));
    assert_eq!(digest.len(), 1_000_000);

    for (quantile, expected) in [
        (0., 1.),
        (0.1, 100_000.),
        (0.5, 500_000.),
        (0.9, 900_000.),
        (1., 1_000_000.),
    ] {
        assert_eq!(digest.quantile(quantile), expected);
    }
}
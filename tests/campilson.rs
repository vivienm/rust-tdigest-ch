// Source: https://github.com/CamDavidsonPilon/tdigest/blob/master/tests/test_tdigest.py

use rand::Rng;
use tdigest_ch::TDigest;

#[test]
fn uniform() {
    let mut digest = TDigest::new();
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        digest.insert(rng.gen::<f32>());
    }

    for (quantile, tolerance) in [
        (0.5, 0.01),
        (0.1, 0.01),
        (0.9, 0.01),
        (0.01, 0.005),
        (0.99, 0.005),
        (0.001, 0.001),
        (0.999, 0.001),
    ] {
        assert!(
            (digest.quantile(quantile as f64) - quantile).abs() < tolerance,
            "quantile {}",
            quantile
        );
    }
}

#[test]
fn ints() {
    let mut digest = TDigest::from([1.0, 2.0, 3.0]);
    assert!(digest.quantile(0.5) - 2.0 < 0.0001);

    let values = vec![1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0];
    let mut digest = TDigest::from_iter(values.iter().copied());
    assert_eq!(digest.quantile(0.5), 2.0);
    assert_eq!(digest.len(), values.len());

    let mut digest = TDigest::from([1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 4.0, 4.0, 5.0, 5.0]);
    assert_eq!(digest.quantile(0.3), 2.0);
    assert_eq!(digest.quantile(0.4), 2.0);
}

#![cfg(feature = "serde")]

use tdigest_ch::TDigest;

#[test]
fn serialize() {
    let digest = TDigest::from_iter([1.0, 2.0, 3.0, 4.0, 5.0]);
    let serialized = serde_json::to_string(&digest).unwrap();
    assert_eq!(
        serialized,
        "[[0.01,2048,2048],[[1.0,1],[2.0,1],[3.0,1],[4.0,1],[5.0,1]],5,5]"
    );
}

#[test]
fn deserialize() {
    let mut digest: TDigest =
        serde_json::from_str("[[0.01,2048,2048],[[1.0,1],[2.0,1],[3.0,1],[4.0,1],[5.0,1]],5,5]")
            .unwrap();
    assert_eq!(digest.quantile(0.0), 1.0);
    assert_eq!(digest.quantile(0.5), 3.0);
    assert_eq!(digest.quantile(1.0), 5.0);
}

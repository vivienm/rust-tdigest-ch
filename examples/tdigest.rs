use ch_tdigest::TDigest;

fn main() {
    let mut digest = TDigest::new();
    digest.insert(1.0);
    digest.insert(2.0);
    digest.insert(3.0);
    println!("{:?}", digest.quantile(0.5));
}

# tdigest-ch

A Rust implementation of [ClickHouse t-digest][ch-tdigest] data structure
([source][ch-tdigest-src]).

The [t-digest][tdigest-paper] data structure is designed around computing
accurate quantile estimates from streaming data. Two t-digests can be merged,
making the data structure ideal for map-reduce settings.

[API documentation][api-docs]

[tdigest-paper]: https://github.com/tdunning/t-digest/blob/main/docs/t-digest-paper/histo.pdf
[ch-tdigest]: https://clickhouse.com/docs/en/sql-reference/aggregate-functions/reference/quantiletdigest/
[ch-tdigest-src]: https://github.com/ClickHouse/ClickHouse/blob/5e34f48a181744a9f9241e3da0522eeaf9c68b84/src/AggregateFunctions/QuantileTDigest.h
[api-docs]: https://vivienm.github.io/rust-tdigest-ch/tdigest_ch/

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

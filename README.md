# toy payment engine

Some assumptions that were used when building that

* Account ids are unique
* Transaction ids are unique per account
* Dispute only works on deposits since we need to hold funds in dispute
* Chargeback cannot occur if it leaves account balance negative

Main logic is implemented in `account.rs`, it describes `Account` struct that can be modified using actions defined as an algebra.
Operations like deposit/withdrawal etc. are encoded in `Transaction` enum after being from raw csv and validated.

Financial computations are handled by `rust_decimal` crate that provides operations on decimals without round-off errors unlike typical floating precision types. While `rust_decimal` is not arbitrary-precision it should cover the case of 4 digits precision and even might be a bit overkill for that if we want to save space.

To run

```bash
cargo run -- <path/to/input/file.csv>
```

To run tests

```bash
cargo test
```

Built & tested with `1.61.0-x86_64-unknown-linux-gnu`

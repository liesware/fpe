# fpe — Liesware fork

Upstream: [str4d/fpe](https://github.com/str4d/fpe). The
[crates.io package](https://crates.io/crates/fpe) follows upstream release numbering.

This crate contains pure-Rust implementations of format-preserving encryption
algorithms.

The following algorithms are implemented:
- FF1 (specified in [NIST Special Publication 800-38G](http://dx.doi.org/10.6028/NIST.SP.800-38G), with compatibility updates from the [2025 second public draft of Revision 1](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-38Gr1.2pd.pdf)).

This crate does not implement FF3 or FF3-1.

## Liesware fork 0.2.0

This fork is based on upstream 0.7.0, commit
`e2654af6072f4d83588a29064f97ccbdb04eb34e`. Fork versions and tags are separate
from the upstream/crates.io release numbering.

This crate requires Rust **1.85** or greater (edition 2024), `cipher 0.5.1` or a
compatible 0.5 release, and `cbc 0.2`. Tests and examples use **AES 0.9**.
The `alloc` implementation uses `num-bigint 0.5`.

### Migrating a consumer

Update the FPE Git reference to the approved fork release and update `aes` from
0.8 to 0.9. Any direct `cipher` dependency used with FPE must move from 0.4 to
0.5. The old AES type implements a different trait version and cannot be passed
to this FPE; no trait adapter is provided.

The usual `FF1::new`, `encrypt`, and `decrypt` calls and the `NumeralString` and
`Operations` contracts retain their shape. Custom cipher implementations or
bounds must migrate to `BlockCipherEncrypt`; code using the old mutable block
mode traits must use `BlockModeEncrypt`. RustCrypto arrays now use `Array`
instead of `GenericArray`. Incorrect key lengths still panic in `FF1::new`.

See [`examples/aes09.rs`](examples/aes09.rs) for a compilable consumer:

```sh
cargo run --locked --example aes09
```

### Preserved fork behavior and compatibility exception

FF1 parameters use integer arithmetic without a direct `libm` dependency.
Tweaks longer than `u32::MAX` bytes are rejected with `TweakTooLong`; padding
avoids intermediate overflow. The 672 frozen baseline vectors retain their
ciphertexts and pass old/new decryption in both directions.

**Compatibility exception:** the prior 62-bit fixed logarithm could underestimate
FF1's byte count for very large valid inputs. Version 0.2.0 corrects its precision;
for example radix 56 and length 3,533,896,755 require one more byte than the old
calculation. Ciphertexts affected by that old defect may differ and must be
migrated with the old implementation before changing versions. No legacy mode
is provided. This exception was explicitly approved during the migration.
See the [integer calculation certificate](docs/integer-parameters.md) and
[migration record](docs/upstream-0.7.0.md).

### Features and validation

The default feature is `std`, which enables `alloc`. Use
`default-features = false, features = ["alloc"]` for allocation-backed numerals
in a `no_std` consumer, or disable all features to provide your own numeral
implementation without allocation. The library does not directly depend on AES.

```sh
cargo test --locked
cargo test --locked --no-default-features --features alloc
cargo test --locked --no-default-features
python3 scripts/check-compatibility.py --toolchain 1.85.0
python3 scripts/reproduce-precision.py
```

The compatibility script builds the original commit and current implementation
in separate processes and exchanges the frozen fixtures. It never rewrites the
expected vectors. Both `no_std` configurations are also checked on
`wasm32-unknown-unknown`; CI runs native 32-bit tests on i686 Linux.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

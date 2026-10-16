# Fork 0.2.0 migration record

## Baseline and preservation inventory

- Fork tag `v0.1.0` resolves to `5a221de5fdd75e8c96e3ecbf8f46f2f18a2460d7`.
- Compare its patch against parent `913e533` and upstream target
  `e2654af6072f4d83588a29064f97ccbdb04eb34e` (upstream 0.7.0).
- Preserve integer FF1 parameter calculation without direct `libm`, tweak length
  rejection and `TweakTooLong`, padding, integer/BigUint tests, feature-gated
  tests and doctests, and architecture-correct benchmark types.
- Baseline tests: 15 unit tests and 1 doctest passed on Rust 1.98.0/macOS arm64.
- Baseline dependencies: AES 0.8.4, cipher 0.4.4, CBC 0.1.2, num-bigint 0.4.8,
  num-integer 0.1.46, num-traits 0.2.19.
- `tests/fixtures/v0.1.0.tsv` contains 672 deterministic synthetic-key fixtures,
  generated and verified before integration, covering AES-128/192/256, radices
  2/3/10/16/36/256/65535/65536, minimum and odd/even lengths, leading zeros,
  binary byte lengths 3/4/5/48, and tweak lengths 0/1/15/16/17/31/32/33.
- Baseline fixture SHA-256:
  `2baff05a919c648887c54f133c95a022fcb815996430336d18f6eb40a6ed4041`.
  Numerals are decimal CSV, keys/tweaks are hexadecimal. For binary rows the
  CSV values are bytes interpreted in the public little-endian bit order.

## Authorized compatibility exception

Investigation before the merge found a pre-existing precision defect in the
62-bit fixed logarithm. For radix 56 and `v = 1766948378`, legal input lengths
`n = 3533896755` and `3533896756` produce `b = 1282662045` on the baseline,
but the FF1 formula requires `1282662046`.

An outward-rounded integer logarithm interval independently certifies
`floor(log2(56) * 2^96) = 460106059542792763718838584228`.
Both interval endpoints, multiplied by v and divided by `8 * 2^96`, have floor
1282662045. A separate 90-digit Decimal calculation gives
`log256(56^v) = 1282662045.00000000002634067677606886216233629762571064258...`.
The old `fixed_log2(56)` is 26781697498098525900 in units of `2^-62`.

The user explicitly authorized correcting this defect and documenting the
compatibility exception. Ciphertexts generated with the incorrect old parameter
may change; universal compatibility with the baseline is therefore not claimed.
No multi-gigabyte ciphertext was generated during this investigation. Frozen
fixtures remain unchanged; this is a parameter-level counterexample.

## Integration and API

The work branch is `codex/update-upstream-0.7.0`. Commit `d353e2d` records
fixtures before integration. The upstream merge retains both histories; the
content conflicts in Cargo.toml and ff1.rs were resolved by adopting the new
cipher/CBC APIs while retaining the fork's validations and allocation gates.
The fork benchmark type fix remains in place.

The package is fork version 0.2.0, edition 2024, MSRV 1.85. The library manifest
requests cipher 0.5.1, CBC 0.2, and num-bigint 0.5; dev tests use AES 0.9.
The upstream lockfile is retained, changing only the package version and
removing FPE's libm dependency and its now-unneeded package entry. Its concrete
versions include cipher 0.5.2, CBC 0.2.1 and AES 0.9.2.

The public FF1 calls and numeral traits keep their shape. Cipher implementers
and consumers must migrate incompatible RustCrypto traits/AES types. No adapter
is introduced. PRF forks explicitly restore the CBC IV, buffer and partial
position, with dedicated tests for all prefix lengths 0..=64.

## Validation record

Local environment: macOS arm64, Rust 1.85.0 and stable Rust 1.98.0. The WASI
runtime is Wasmtime 49.0.0 via an isolated Python installation in /tmp.

| Check | Result |
| --- | --- |
| Default and alloc-only host tests, both Rust versions | 24 unit tests, 1 fixture test (672 vectors), 2 validation tests and 1 doctest per configuration |
| No-feature host tests, both Rust versions | 8 unit tests and 1 gated doctest per configuration |
| Old -> new -> old independent process exchange | All 672 fixtures decrypt and re-encrypt exactly |
| Integer parameter calculation | Full legal radix/half-length certificate; original BigUint oracle; adjacent boundaries; large regression case |
| wasm32-unknown-unknown no_std checks | Both no-feature and alloc-only configurations, Rust 1.85.0 and 1.98.0 |
| WASI 32-bit default and alloc-only tests | 23 unit tests, all 672 fixtures, 2 validation tests per configuration |
| WASI 32-bit no-feature tests | 7 unit tests; the excess-u32 tweak/numeral cases are intentionally 64-bit-only |
| Minimal external consumer | Compiled and ran with Rust 1.85, cipher exactly 0.5.1, CBC exactly 0.2.0 and AES exactly 0.9.2 |
| Formatting, Clippy and rustdoc | Rust 1.85 formatting; all-target default and lib/tests reduced-feature Clippy on both toolchains; rustdoc with warnings denied |
| Benchmarks and AES 0.9 example | Compiled benchmarks and ran example on both toolchains |
| Workflow syntax and whitespace | YAML parsed; git diff --check |

Host matrix commands are the test/Clippy commands in CI, using `--locked` and
both toolchains. The compatibility command is
`python3 scripts/check-compatibility.py --toolchain 1.85.0`; it builds the old
consumer from the immutable baseline SHA and its separate checked-in lockfile.
The original fixture SHA-256 above remains unchanged.

For local 32-bit execution, the Rust 1.85 `wasm32-wasip1` test binaries were run
with a Wasmtime WASI runner and `--test-threads=1`. Proptest's subprocess and
Criterion's rayon defaults are disabled only on WASI; normal host defaults are
unchanged. WASI tests do not execute target doctests. CI additionally declares
native i686 Linux tests, Windows/macOS/Linux host tests and the MSRV/stable
matrix. Those remote jobs have not been executed from this local checkout.

No multi-gigabyte encryption/decryption was attempted. The large-input defect
and its correction are verified at parameter level with exact integer bounds.
No Vectis files or other consumers were changed. The pre-existing untracked
.DS_Store is left untouched. v0.1.0 is unchanged; v0.2.0 has not been created or
published and requires separate release approval. No registry publication is
part of this change.

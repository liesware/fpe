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

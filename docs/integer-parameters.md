# Integer FF1 byte-count calculation

For a non-power-of-two radix r and positive half length v,
`b = ceil(log256(r^v)) = floor(v * log2(r) / 8) + 1`.
For powers of two, the logarithm is an exact integer and the implementation uses
`ceil(v * log2(r) / 8)` directly with a u64 intermediate.

## Error bound without floating point

The runtime computes a lower logarithm approximation L with 96 fractional bits.
Repeated squaring uses 112 fractional working bits (S = 2^112). The normalized
initial value `r / 2^floor(log2(r))` is represented exactly.

At each iteration, z lies in [1, 2). The next normalized value is
`floor(S * z^2 / 2^d) / S`, where d is 0 or 1 according to whether the squared
value reaches 2. Its absolute downward rounding error is less than 1/S, and its
logarithmic error is less than `log2(1 + 1/S) < 2/S`.

Unrolling the logarithmic recurrence weights iteration j's rounding error by
2^-j. Their sum is strictly less than 2/S. After 96 iterations, the unrecorded
normalized logarithm contributes less than 2^-96. Consequently:

```
L / 2^96 <= log2(r) < L / 2^96 + 2^-96 + 2^-111
                         < (L + 2) / 2^96
```

`square_shift_112` computes the required square and shift exactly using 64-bit
limbs, u128 products and retained carries. Its inputs are below 2^113, so its
intermediate values fit u128. It is independently tested against BigUint.
To multiply L by v, the implementation splits off L's integer part: the
fractional product needs at most 96 + 31 = 127 bits and fits u128.

## Finite exhaustive certificate

Legal input lengths satisfy n <= 2^32 - 1, hence v = ceil(n/2) <= 2^31.
For each non-power-of-two radix in 2..=65536, the test
`certify_byte_count_for_every_legal_radix_and_half_length` computes exactly:

```
sum(v=0..2^31, floor(v * L       / 2^99))
sum(v=0..2^31, floor(v * (L + 2) / 2^99))
```

The sums use BigUint and Euclidean floor-sum reduction. The second summand is
never smaller than the first for any v. Equality of the sums therefore proves
equality at EVERY legal v. Together with the logarithm interval this certifies
the runtime byte count, rather than just sampling lengths. The v=0 summand is
zero and is included only to simplify the reduction.

Power-of-two radices use exact arithmetic and do not need this certificate.
The original BigUint power oracle remains, supplemented by adjacent byte
boundaries and sparse larger exponents. The floor-sum helper itself is checked
against direct enumeration on small inputs.

## Previous precision defect and compatibility

The previous 62-bit implementation underestimates b at radix 56,
v=1766948378: 1282662045 instead of 1282662046. These correspond to valid n
values 3533896755 and 3533896756. The correction was explicitly authorized;
compatibility is not promised for baseline ciphertexts affected by the old
rounding defect. See `upstream-0.7.0.md` and run
`python3 scripts/reproduce-precision.py` for independent integer-interval evidence.
No expected ciphertext fixture was changed to accommodate this correction.

On 32-bit systems, a half-string byte length larger than usize cannot be
represented by the public byte-buffer interface; such multi-gigabyte operations
are not made addressable by this certificate. Limits and padding are tested
without allocating oversized buffers. The full-domain mathematical certificate
is run on both pointer widths.

#!/usr/bin/env python3
"""Independently reproduce the baseline parameter defect with integer intervals."""


def logarithm_prefix(radix, fractional_bits, working_bits):
    integer = radix.bit_length() - 1
    scale = 1 << working_bits
    low = high = radix << (working_bits - integer)
    prefix = integer << fractional_bits
    for bit in range(fractional_bits - 1, -1, -1):
        low = low * low // scale
        high = (high * high + scale - 1) // scale
        if low >= 2 * scale:
            low //= 2
            high = (high + 1) // 2
            prefix |= 1 << bit
        else:
            assert high < 2 * scale, "Increase precision to disambiguate this bit"
    return prefix


def main():
    radix, v = 56, 1766948378
    integer = radix.bit_length() - 1
    y = radix << (62 - integer)
    old_log = integer << 62
    for bit in range(61, -1, -1):
        y = y * y >> 62
        if y >= 1 << 63:
            y >>= 1
            old_log |= 1 << bit
    prefix = logarithm_prefix(radix, 96, 256)
    lower = v * prefix // (1 << 99)
    upper = v * (prefix + 1) // (1 << 99)
    assert lower == upper
    old_b = v * old_log // (1 << 65) + 1
    exact_b = lower + 1
    assert (old_b, exact_b) == (1282662045, 1282662046)
    assert 2 * v <= (1 << 32) - 1
    print(f"radix={radix}, v={v}, legal n={2*v-1} or {2*v}")
    print(f"baseline b={old_b}; certified exact b={exact_b}")
    print(f"floor(log2(radix) * 2^96)={prefix}")


if __name__ == "__main__":
    main()

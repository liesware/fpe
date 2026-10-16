//! Exact arithmetic checks, including a finite certificate for every legal radix/v.
use super::{FIXED_LOG_BITS, Radix, fixed_log2, square_shift_112};
use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

// Sum floor((a*i+b)/m), i=0..n-1, by Euclidean reduction. BigUint
// deliberately avoids using the runtime's fixed-width multiply implementation.
fn floor_sum(n: u64, m: u128, a: u128) -> BigUint {
    let (mut n, mut m, mut a, mut b) = (
        BigUint::from(n),
        BigUint::from(m),
        BigUint::from(a),
        BigUint::zero(),
    );
    let mut sum = BigUint::zero();
    loop {
        sum += (&n - 1u32) * &n * (&a / &m) / 2u32 + &n * (&b / &m);
        a %= &m;
        b %= &m;
        let y = &a * &n + &b;
        if y < m {
            return sum;
        }
        n = &y / &m;
        b = y % &m;
        core::mem::swap(&mut m, &mut a);
    }
}

#[test]
fn euclidean_floor_sum_matches_direct_sum() {
    for n in 1..=20 {
        for m in 1..=20 {
            for a in 0..=20 {
                let expected: u128 = (0..n).map(|i| a * u128::from(i) / m).sum();
                assert_eq!(floor_sum(n, m, a), BigUint::from(expected));
            }
        }
    }
}

#[test]
fn fixed_width_square_matches_biguint() {
    let check = |y: u128| {
        let big = BigUint::from(y);
        let expected = ((&big * &big) >> 112usize).to_u128().unwrap();
        assert_eq!(square_shift_112(y), expected, "y={y}");
    };
    for y in [0, 1, u64::MAX as u128, 1 << 112, (1 << 113) - 1] {
        check(y);
    }
    let mut state = 0x1234_5678_9abc_def0u128;
    for _ in 0..4096 {
        state = state.wrapping_mul(0xda94_2042_e4dd_58b5).wrapping_add(1);
        check(state & ((1 << 113) - 1));
    }
}

#[test]
fn corrected_large_domain_parameter() {
    // Independently certified 96-bit logarithm prefix; no huge allocation.
    let radix = Radix::from_u32(56).unwrap();
    assert_eq!(fixed_log2(56), 460_106_059_542_792_763_718_838_584_228);
    assert_eq!(radix.calculate_b(1_766_948_378), 1_282_662_046);
    assert_eq!(radix.calculate_b(1_766_948_377), 1_282_662_045);
    assert_eq!(radix.calculate_b(1_766_948_379), 1_282_662_046);
}

#[test]
fn certify_byte_count_for_every_legal_radix_and_half_length() {
    // See docs/integer-parameters.md. For each non-power-of-two radix,
    // L/2^96 <= log2(radix) < (L+2)/2^96. Equality of these two
    // sums proves that no v in 1..=2^31 crosses a byte boundary between
    // the bounds. This checks ALL half lengths, without enumerating them.
    let n = (1u64 << 31) + 1;
    let denominator = 1u128 << (FIXED_LOG_BITS + 3);
    for radix in 2u32..=65536 {
        if radix.is_power_of_two() {
            continue; // Exact log2 is an integer, handled by a separate path.
        }
        let lower = fixed_log2(radix);
        assert_eq!(
            floor_sum(n, denominator, lower),
            floor_sum(n, denominator, lower + 2),
            "logarithm interval crosses a byte boundary: radix={radix}"
        );
    }
}

#[test]
fn byte_boundaries_match_biguint_oracle() {
    for radix in [3u32, 5, 7, 10, 36, 56, 255, 257, 65535] {
        let r = Radix::from_u32(radix).unwrap();
        let mut domain = BigUint::one();
        let mut previous = 0;
        for v in 1usize..=1024 {
            domain *= radix;
            let expected = ((&domain - 1u32).bits() as usize).div_ceil(8);
            if expected != previous {
                for adjacent in v.saturating_sub(1).max(1)..=v + 1 {
                    let max = BigUint::from(radix).pow(adjacent as u32) - 1u32;
                    assert_eq!(r.calculate_b(adjacent), (max.bits() as usize).div_ceil(8));
                }
            }
            previous = expected;
        }
        for v in [4095, 4096, 4097, 65535, 65536, 65537] {
            let max = BigUint::from(radix).pow(v as u32) - 1u32;
            assert_eq!(r.calculate_b(v), (max.bits() as usize).div_ceil(8));
        }
    }
}

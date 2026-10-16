#![cfg(feature = "alloc")]
use aes::Aes128;
use fpe::ff1::{FF1, FlexibleNumeralString, NumeralStringError};

#[test]
fn rejects_invalid_radices_and_numerals_in_both_directions() {
    for radix in [0, 1, 65537, u32::MAX] {
        assert!(FF1::<Aes128>::new(&[0; 16], radix).is_err());
    }
    let ff = FF1::<Aes128>::new(&[0; 16], 10).unwrap();
    let invalid = FlexibleNumeralString::from(vec![0, 1, 2, 3, 4, 10]);
    assert_eq!(
        ff.encrypt(&[], &invalid).err(),
        Some(NumeralStringError::InvalidForRadix(10))
    );
    assert_eq!(
        ff.decrypt(&[], &invalid).err(),
        Some(NumeralStringError::InvalidForRadix(10))
    );
}

#[test]
fn enforces_domain_threshold_for_every_fixture_radix() {
    for (radix, min_len) in [
        (2, 20),
        (3, 13),
        (10, 6),
        (16, 5),
        (36, 4),
        (256, 3),
        (65535, 2),
        (65536, 2),
    ] {
        let ff = FF1::<Aes128>::new(&[0; 16], radix).unwrap();
        let short = FlexibleNumeralString::from(vec![0; min_len - 1]);
        let expected = NumeralStringError::TooShort {
            ns_len: min_len - 1,
            min_len,
        };
        assert_eq!(ff.encrypt(&[], &short).err(), Some(expected));
        assert_eq!(ff.decrypt(&[], &short).err(), Some(expected));
        let valid = FlexibleNumeralString::from(vec![0; min_len]);
        let ct = ff.encrypt(&[], &valid).unwrap();
        assert_eq!(Vec::from(ff.decrypt(&[], &ct).unwrap()), vec![0; min_len]);
    }
}

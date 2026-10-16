//! Minimal consumer using AES 0.9 directly, without trait adapters.
use aes::Aes256;
use fpe::ff1::{FF1, FlexibleNumeralString};

fn main() {
    let ff = FF1::<Aes256>::new(&[0; 32], 10).unwrap();
    let plaintext = FlexibleNumeralString::from(vec![0, 1, 2, 3, 4, 5]);
    let ciphertext = ff.encrypt(b"example", &plaintext).unwrap();
    let recovered = ff.decrypt(b"example", &ciphertext).unwrap();
    assert_eq!(Vec::from(recovered), Vec::from(plaintext));
}

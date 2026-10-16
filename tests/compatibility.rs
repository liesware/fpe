#![cfg(feature = "alloc")]
#[allow(dead_code)]
#[path = "support/compatibility.rs"]
mod compatibility;

#[test]
fn frozen_v010_ciphertexts() {
    let output = compatibility::verify_and_encrypt(include_str!("fixtures/v0.1.0.tsv"));
    assert_eq!(output.lines().count(), 672);
}

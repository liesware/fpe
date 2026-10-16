//! Fixture transport for independent old/new compatibility checks.
#[path = "../tests/support/compatibility.rs"]
mod compatibility;

fn main() {
    use std::io::{Read, Write};
    let output = if std::env::args().nth(1).as_deref() == Some("--generate-baseline") {
        compatibility::generate_baseline()
    } else {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
        compatibility::verify_and_encrypt(&input)
    };
    std::io::stdout().write_all(output.as_bytes()).unwrap();
}

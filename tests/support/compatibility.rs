use aes::{Aes128, Aes192, Aes256};
use fpe::ff1::{BinaryNumeralString, FlexibleNumeralString, FF1};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn numerals(s: &str) -> Vec<u16> {
    s.split(',').map(|n| n.parse().unwrap()).collect()
}

fn digits(ns: &[u16]) -> String {
    ns.iter().map(u16::to_string).collect::<Vec<_>>().join(",")
}

// This source is compiled separately against the old and new FPE/AES versions.
// No cipher traits or version adapters cross the process boundary.
fn crypt(
    key: &[u8],
    radix: u32,
    tweak: &[u8],
    input: &[u16],
    binary: bool,
    decrypt: bool,
) -> Vec<u16> {
    macro_rules! run {
        ($aes:ty) => {{
            let ff = FF1::<$aes>::new(key, radix).unwrap();
            if binary {
                assert_eq!(radix, 2);
                let bytes: Vec<u8> = input.iter().map(|&n| u8::try_from(n).unwrap()).collect();
                let ns = BinaryNumeralString::from_bytes_le(&bytes);
                let result = if decrypt {
                    ff.decrypt(tweak, &ns)
                } else {
                    ff.encrypt(tweak, &ns)
                }
                .unwrap();
                result.to_bytes_le().into_iter().map(u16::from).collect()
            } else {
                let ns = FlexibleNumeralString::from(input.to_vec());
                let result = if decrypt {
                    ff.decrypt(tweak, &ns)
                } else {
                    ff.encrypt(tweak, &ns)
                }
                .unwrap();
                Vec::from(result)
            }
        }};
    }
    match key.len() {
        16 => run!(Aes128),
        24 => run!(Aes192),
        32 => run!(Aes256),
        _ => panic!("invalid synthetic key length"),
    }
}

pub fn verify_and_encrypt(input: &str) -> String {
    let mut output = String::new();
    for line in input.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7);
        let key = unhex(fields[2]);
        let radix = fields[3].parse().unwrap();
        let tweak = unhex(fields[4]);
        let pt = numerals(fields[5]);
        let expected = numerals(fields[6]);
        let binary = fields[1] == "binary";
        assert!(binary || fields[1] == "flexible");
        assert_eq!(
            crypt(&key, radix, &tweak, &expected, binary, true),
            pt,
            "decrypt {}",
            fields[0]
        );
        let ct = crypt(&key, radix, &tweak, &pt, binary, false);
        assert_eq!(ct, expected, "encrypt {}", fields[0]);
        output.push_str(&fields[..6].join("\t"));
        output.push('\t');
        output.push_str(&digits(&ct));
        output.push('\n');
    }
    assert!(!output.is_empty());
    output
}

// Run only on 5a221de to establish fixtures; never update expected results on the
// migrated implementation in response to a mismatch.
pub fn generate_baseline() -> String {
    let mut output = String::from("# Synthetic keys only. Generated on 5a221de5fdd75e8c96e3ecbf8f46f2f18a2460d7\n# id\trepresentation\tkey_hex\tradix\ttweak_hex\tplaintext\tciphertext\n");
    for key_len in [16, 24, 32] {
        let key: Vec<u8> = (0..key_len).map(|i| (i * 7 + key_len) as u8).collect();
        for radix in [2u32, 3, 10, 16, 36, 256, 65535, 65536] {
            let mut min_len = 2;
            let mut domain = u64::from(radix).pow(min_len);
            while domain < 1_000_000 {
                domain *= u64::from(radix);
                min_len += 1;
            }
            for len in [min_len as usize, min_len as usize + 1, 81] {
                for tweak_len in [0, 1, 15, 16, 17, 31, 32, 33] {
                    let tweak: Vec<u8> = (0..tweak_len).map(|i| (i * 11) as u8).collect();
                    let mut pt: Vec<u16> = (0..len)
                        .map(|i| ((i as u32 * 1237 + 19) % radix) as u16)
                        .collect();
                    pt[0] = 0;
                    let ct = crypt(&key, radix, &tweak, &pt, false, false);
                    output.push_str(&format!(
                        "aes{}-r{radix}-n{len}-t{tweak_len}\tflexible\t{}\t{radix}\t{}\t{}\t{}\n",
                        key_len * 8,
                        hex(&key),
                        hex(&tweak),
                        digits(&pt),
                        digits(&ct)
                    ));
                }
            }
        }
        for len in [3, 4, 5, 48] {
            for tweak_len in [0, 1, 15, 16, 17, 31, 32, 33] {
                let tweak: Vec<u8> = (0..tweak_len).map(|i| (i * 11) as u8).collect();
                let mut pt: Vec<u16> = (0..len).map(|i| ((i * 37 + 19) % 256) as u16).collect();
                pt[0] = 0;
                let ct = crypt(&key, 2, &tweak, &pt, true, false);
                output.push_str(&format!(
                    "aes{}-binary-bytes{len}-t{tweak_len}\tbinary\t{}\t2\t{}\t{}\t{}\n",
                    key_len * 8,
                    hex(&key),
                    hex(&tweak),
                    digits(&pt),
                    digits(&ct)
                ));
            }
        }
    }
    output
}

#!/usr/bin/env python3
"""Compile independent old/new binaries and exchange frozen FF1 fixtures.

Requires git, Cargo, and the selected toolchain. Never writes expected vectors.
"""
import argparse
import io
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "5a221de5fdd75e8c96e3ecbf8f46f2f18a2460d7"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain", default="1.85.0")
    args = parser.parse_args()
    cargo = ["cargo", "+" + args.toolchain]
    with tempfile.TemporaryDirectory(prefix="fpe-compat-") as tmp:
        work = Path(tmp)
        old = work / "old"
        old.mkdir()
        archive = subprocess.check_output(["git", "archive", BASELINE], cwd=ROOT)
        with tarfile.open(fileobj=io.BytesIO(archive)) as files:
            files.extractall(old, filter="data")
        consumer = work / "consumer"
        (consumer / "src").mkdir(parents=True)
        # This manifest contains only the OLD FPE and AES. The new crate is
        # compiled by its own example command, in an entirely separate process.
        (consumer / "Cargo.toml").write_text(
            '[package]\nname = "fpe-compatibility-baseline"\n'
            'version = "0.0.0"\nedition = "2021"\n'
            '[dependencies]\naes = "=0.8.4"\n'
            f'fpe = {{ path = "{old.as_posix()}" }}\n'
        )
        shutil.copyfile(ROOT / "tests/support/compatibility.rs", consumer / "src/compatibility.rs")
        (consumer / "src/main.rs").write_text(
            '#[allow(dead_code)] mod compatibility;\nfn main() {\n'
            'use std::io::{Read, Write};\nlet mut input = String::new();\n'
            'std::io::stdin().read_to_string(&mut input).unwrap();\n'
            'std::io::stdout().write_all('
            'compatibility::verify_and_encrypt(&input).as_bytes()).unwrap();\n}\n'
        )
        shutil.copyfile(ROOT / "tests/fixtures/baseline-consumer.lock", consumer / "Cargo.lock")
        old_command = cargo + ["run", "--quiet", "--locked", "--manifest-path", str(consumer / "Cargo.toml")]
        new_command = cargo + ["run", "--quiet", "--locked", "--example", "compatibility"]
        data = (ROOT / "tests/fixtures/v0.1.0.tsv").read_bytes()
        for command in [old_command, new_command, old_command]:
            data = subprocess.check_output(command, input=data, cwd=ROOT)
        expected = b"\n".join(
            line for line in (ROOT / "tests/fixtures/v0.1.0.tsv").read_bytes().splitlines()
            if line and not line.startswith(b"#")
        ) + b"\n"
        assert data == expected, "Old/new ciphertext exchange changed the frozen fixtures"
        print("672 fixtures passed old -> new -> old, including decryption in each process")


if __name__ == "__main__":
    main()

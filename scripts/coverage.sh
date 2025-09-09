#!/bin/bash
# Script untuk generate coverage dan kirim ke SonarQube

set -e  # Exit on any error

echo "==== Membersihkan direktori coverage ===="
rm -rf target/coverage
mkdir -p target/coverage

echo "==== Menjalankan tests dengan coverage ===="
# Menggunakan cargo-llvm-cov (recommended untuk Rust)
cargo llvm-cov --all-features --workspace --lcov --output-path target/coverage/lcov.info

# Alternative menggunakan tarpaulin
# cargo tarpaulin --out Xml --out Html --output-dir target/coverage/ --all-features

echo "==== Coverage report berhasil dibuat ===="
echo "File coverage: target/coverage/lcov.info"

echo "==== Menjalankan SonarQube Scanner ===="
# Pastikan sonar-scanner sudah terinstall dan ada di PATH
sonar-scanner

echo "==== Selesai ===="
@echo off
REM Script untuk generate coverage dan kirim ke SonarQube

echo ==== Membersihkan direktori coverage ====
if exist target\coverage rmdir /s /q target\coverage
mkdir target\coverage

echo ==== Menjalankan tests dengan coverage ====
REM Menggunakan cargo-llvm-cov (recommended untuk Rust)
cargo llvm-cov --all-features --workspace --lcov --output-path target/coverage/lcov.info

REM Alternative menggunakan tarpaulin
REM cargo tarpaulin --out Xml --out Html --output-dir target/coverage/ --all-features

echo ==== Coverage report berhasil dibuat ====
echo File coverage: target/coverage/lcov.info

echo ==== Menjalankan SonarQube Scanner ====
REM Pastikan sonar-scanner sudah terinstall dan ada di PATH
sonar-scanner.bat

echo ==== Selesai ====
pause
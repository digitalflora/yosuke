cd "$(dirname "$0")"
cargo build --release --bin server
cargo build --release --bin client --target x86_64-pc-windows-gnu
mv target/x86_64-pc-windows-gnu/release/client.exe target/release/stub.dat

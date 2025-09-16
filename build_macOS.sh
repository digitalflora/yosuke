cd "$(dirname "$0")"
cd ./server
cargo bundle --release
cd ../client
cargo build --release --target x86_64-pc-windows-gnu
cd ../
mv target/x86_64-pc-windows-gnu/release/client.exe target/release/stub.dat

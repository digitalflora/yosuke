cd "$(dirname "$0")"
rm -r target/release/bundle/linux/
cd ./server
cargo build --release
cd ../client
cargo build --release --target x86_64-pc-windows-gnu
cd ../
mkdir -p target/release/bundle/linux
mv target/x86_64-pc-windows-gnu/release/client.exe target/release/bundle/linux/stub.dat
echo Check: target/release/bundle/linux/Yosuke

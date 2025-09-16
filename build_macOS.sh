cd "$(dirname "$0")"
rm -r target/release/bundle/osx/
cd ./server
cargo bundle --release # creates bundle/osx
cd ../client
cargo build --release --target x86_64-pc-windows-gnu
cd ../
mv target/x86_64-pc-windows-gnu/release/client.exe target/release/bundle/osx/stub.dat
open -R target/release/bundle/osx/Yosuke.app
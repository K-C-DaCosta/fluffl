
(Documentation is a work in progress)

To build on wasm just:
```
cargo build --release --target=wasm32-unknown-unknown --no-default-features --features='web'
wasm-bindgen ./target/wasm32-unknown-unknown/release/brick_demo.wasm --out-dir ../../wasm_bins/examples/brick_demo --target web
```
To run just copy paste the index.html into the ../examples/brick_demo directory







1. compile examplelib to wasm: `cargo build -p examplelib --target wasm32-unknown-unknown --release`
2. run wasmrunner pointing to wasm file to load: `cargo run -p wasmrunner -- ./target/wasm32-unknown-unknown/release/examplelib.wasm`


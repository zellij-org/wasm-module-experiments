# Testing Strider

As a very temporary solution, run the following:

```sh
rustup override set nightly
rustup target add wasm32-wasi
cd module
cargo build
cd ..
cargo run loader
```

# Tasks
- Represent KeyModifiers with booleans instead of bits

# obfus-rs

<p align="center">
  <img src="docs/images/banner.png">
</p>

Compile-time string obfuscation library for Rust.

> **Note:** This project is under active development. More features and improvements are planned.

## Features

- Compile-time string obfuscation
- Runtime string deobfuscation
- XOR-based encryption
- SplitMix64-based keystream generation
- Seed generation based on source location
- No external dependencies
- UTF-8 support

## Example

```rust
use obfus::obfstr;

fn main() {
    let value = obfstr!("Hello, World!");

    assert_eq!(value, "Hello, World!");
}
```

## Usage

Use `obfstr!` to obfuscate string literals at compile time. The encrypted data is decrypted at runtime when the expression is evaluated.

```rust
let message = obfstr!("Sensitive string");

println!("{message}");
```

## Testing

Run the test suite with Cargo:

```bash
cargo test
```

## Disclaimer

This library is intended to make static string analysis more difficult. It is not intended for cryptographic security and does not guarantee that strings cannot be recovered at runtime.
# serde_hash

A Rust library for seamlessly integrating HashIds with Serde serialization and deserialization. This library provides a convenient way to obfuscate numeric IDs in your JSON output without changing your application's internal data structures.

## Features

- Automatically convert numeric IDs to hash strings during serialization
- Transparently decode hash strings back to numeric IDs during deserialization
- Configurable hash generation with customizable salt, minimum length, and character set
- Simple attribute-based field marking with `#[hash]`
- Secure random salt generation

## Installation

Add `serde_hash` and `serde_hash_derive` to your `Cargo.toml`:

```toml
[dependencies]
serde_hash = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # If using JSON serialization
```

## Usage

> ### Note:
> The `#[hash]` attribute is only compatible with unsigned integer types such as `u64`, `u32`, `usize`, etc. It cannot be used with floating-point (`f32`, `f64`) or signed integer (`i32`, `i64`) types.

### Basic Example

To use `serde_hash` with your data structures, add the `HashIds` derive macro and mark the fields you want to hash with `#[hash]`:

```rust
use serde_hash::hashids::SerdeHashOptions;
use serde_hash_derive::HashIds;

// Define your data structure with the HashIds derive macro
#[derive(HashIds, Debug)]
pub struct User {
    // Mark numeric ID fields with #[hash]
    #[hash]
    pub id: u64,
    // Regular fields remain unchanged
    pub name: String,
    pub age: u8,
}

fn main() {
    // Configure hash ID generation settings
    SerdeHashOptions::new()
        .with_salt("my-secret-salt")
        .with_min_length(10)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // Create a data instance
    let user = User {
        id: 158674,
        name: "Dan Smith".to_string(),
        age: 47,
    };

    // Serialize to JSON - the ID will be automatically hashed
    let json_string = serde_json::to_string(&user).unwrap();
    println!("{:?} -> {}", user, json_string);
    // Output: User { id: 158674, name: "Dan Smith", age: 47 } -> {"id":"qKknODM7Ej","name":"Dan Smith","age":47}
}
```

### Deserialization Example

The library seamlessly handles deserialization, converting the hash strings back to original numeric IDs:

```rust
use serde_hash::hashids::SerdeHashOptions;
use serde_hash_derive::HashIds;

#[derive(HashIds, Debug)]
pub struct User {
    #[hash]
    pub id: u64,
    pub name: String,
    pub age: u8,
}

fn main() {
    // Configure hash ID settings (must match serialization settings)
    SerdeHashOptions::new()
        .with_salt("my-secret-salt")
        .with_min_length(10)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // JSON string with hashed ID
    let json_string = r#"{"id":"qKknODM7Ej","name":"Dan Smith","age":47}"#;
    
    // Deserialize JSON string back to User struct
    let user: User = serde_json::from_str(json_string).unwrap();
    
    // The ID is automatically decoded back to its original numeric value
    println!("Deserialized user: {:?}", user);
    // Output: Deserialized user: User { id: 158674, name: "Dan Smith", age: 47 }
}
```

### Generating Secure Salt

For production use, it's recommended to use a cryptographically secure random salt:

```rust
use serde_hash::{hashids::SerdeHashOptions, salt::generate_salt};
use serde_hash_derive::HashIds;

#[derive(HashIds, Debug)]
pub struct User {
    #[hash]
    pub id: u64,
    pub name: String,
}

fn main() {
    // Generate a secure random salt
    let salt = generate_salt();
    println!("Generated salt: {}", salt);
    
    // Configure hash ID settings with the generated salt
    SerdeHashOptions::new()
        .with_salt(&salt)
        .with_min_length(10)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();
        
    // Use the configured settings for serialization/deserialization
    // ...
}
```

## Configuration Options

You can customize the hash ID generation with the following options:

- **Salt**: A secret string that affects the generated hash. Different salts produce different hashes for the same input.
- **Minimum Length**: The minimum character length of generated hash strings.
- **Alphabet**: The set of characters used in the hash string. Default is alphanumeric.

## Why Use serde_hash?

- **Obfuscation**: Hide your internal database IDs from API consumers
- **Predictability**: Unlike UUID generation, the same ID will always hash to the same string with the same settings
- **Transparency**: Your application code can work with numeric IDs while your API exposes hash strings
- **Simplicity**: Add one derive macro and configure once, no need to manually encode/decode IDs

## License

[MIT License](LICENSE)

---

This project is built with Rust and uses the [hash-ids](https://crates.io/crates/hash-ids) crate for hash ID generation.
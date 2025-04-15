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
> The `#[hash]` attribute is only compatible with unsigned integer types such as `u64`, `u32`, `usize`, etc., and vectors of these types like `Vec<u64>`, `Vec<u32>`, `Vec<usize>`. It cannot be used with floating-point (`f32`, `f64`) or signed integer (`i32`, `i64`) types or their vector equivalents.

## Configuration Options

To customize your hash settings, such as the salt value or the minimum length of the generated hash strings, utilize the `SerdeHashOptions` builder. The following options are available:

| Name       | Default Value            | Description                                     |
|------------|--------------------------|-------------------------------------------------|
| salt       | Generated randomly       | The cryptographic salt used for hash generation |
| min_length | 8                        | Minimum length of the generated hash string     |
| alphabet   | Alphanumeric (a-zA-Z0-9) | Characters used for hash encoding               |

Simpliest example:

```rust
use serde_hash::hashids::SerdeHashOptions;
fn main() {
	SerdeHashOptions::new().build();
}
```

A more complete example:

```rust
use serde_hash::hashids::SerdeHashOptions;
fn main() {
	// Configure the hash ID generator with custom settings
	// This affects how numeric IDs are converted to hash strings
	// All of these fields are optional.
	SerdeHashOptions::new()
		// Set the cryptographic salt for the hash algorithm,
		// By default, this is a randomly generated sequence of characters using the `generate_salt()` function.
		.with_salt("hello world")
		// Ensure generated hashes are at least 10 characters long,
		// This value is 8 by default.
		.with_min_length(10)
		// Define the character set used in the hash encoding (alphanumeric in this case)
		// Below is the default value.
		.with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
		// This will save these values globally and is thread-safe.
		.build();
}
```

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

### Using with Vectors

The `#[hash]` attribute can also be applied to vectors of unsigned integers. When applied to a vector field, each element in the vector will be hashed individually:

```rust
use serde_hash::hashids::SerdeHashOptions;
use serde_hash_derive::HashIds;

#[derive(HashIds, Debug)]
pub struct DataWithVector {
	#[hash]
	pub id: u64,
	pub name: String,
	// Vector of unsigned integers that will be hashed
	#[hash]
	pub values: Vec<u8>,
}

fn main() {
	// Configure hash ID generation settings
	SerdeHashOptions::new()
		.with_salt("my-secret-salt")
		.build();

	// Create a data instance with a vector
	let data = DataWithVector {
		id: 158674,
		name: "Dan Smith".to_string(),
		values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
	};

	// Serialize to JSON - both the ID and each vector element will be hashed
	let json_string = serde_json::to_string(&data).unwrap();
	println!("{:?} -> {}", data, json_string);

	// Deserialize back to the original structure
	let deserialized: DataWithVector = serde_json::from_str(&json_string).unwrap();
	println!("Deserialized: {:?}", deserialized);
}
```

When serialized, each value in the vector will be individually hashed, and during deserialization, each hashed string will be converted back to its original numeric value. This feature is useful when working with collections of IDs that need to be obfuscated in your API responses.

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

## Why Use serde_hash?

- **Obfuscation**: Hide your internal database IDs from API consumers
- **Predictability**: Unlike UUID generation, the same ID will always hash to the same string with the same settings
- **Transparency**: Your application code can work with numeric IDs while your API exposes hash strings
- **Simplicity**: Add one derive macro and configure once, no need to manually encode/decode IDs

## License

[MIT License](LICENSE)

---

This project is built with Rust and uses the [hash-ids](https://crates.io/crates/hash-ids) crate for hash ID generation.
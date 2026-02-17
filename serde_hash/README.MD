# serde_hash

A Rust library for seamlessly integrating HashIds with Serde serialization and deserialization. This library provides a convenient way to obfuscate numeric IDs in your JSON output without changing your application's internal data structures.

## Features

- Extends serde's `Serialize` and `Deserialize` derives -- all serde attributes (`rename`, `alias`, `default`, `skip`, etc.) work alongside hash encoding
- Automatically convert numeric IDs to hash strings during serialization
- Transparently decode hash strings back to numeric IDs during deserialization
- Configurable hash generation with customizable salt, minimum length, and character set
- Simple attribute-based field marking with `#[serde(hash)]`
- Secure random salt generation

## Installation

Add `serde_hash` to your `Cargo.toml`:

```toml
[dependencies]
serde_hash = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # If using JSON serialization
```

## Supported Types

The `hash` attribute is only compatible with unsigned integer types and collections of them. It cannot be used with floating-point (`f32`, `f64`) or signed integer (`i32`, `i64`) types.

| Category                     | Supported Types                                                  |
|------------------------------|------------------------------------------------------------------|
| Unsigned integers            | `u8`, `u16`, `u32`, `u64`, `u128`, `usize`                       |
| Optional unsigned integers   | `Option<u8>`, `Option<u16>`, ..., `Option<usize>`                |
| Vectors of unsigned integers | `Vec<u8>`, `Vec<u16>`, ..., `Vec<usize>`                         |
| Optional vectors             | `Option<Vec<u8>>`, `Option<Vec<u16>>`, ..., `Option<Vec<usize>>` |

## Usage

### Configuration Options

Customize hash settings with `SerdeHashOptions`. Call `.build()` once at startup before any serialization.

| Name       | Default Value            | Description                                     |
|------------|--------------------------|-------------------------------------------------|
| salt       | Generated randomly       | The cryptographic salt used for hash generation |
| min_length | 8                        | Minimum length of the generated hash string     |
| alphabet   | Alphanumeric (a-zA-Z0-9) | Characters used for hash encoding               |

Simplest example:

```rust
use serde_hash::hashids::SerdeHashOptions;
SerdeHashOptions::new().build();
```

A more complete example:

```rust
use serde_hash::hashids::SerdeHashOptions;
SerdeHashOptions::new()
.with_salt("hello world")
.with_min_length(10)
.with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
.build();
```

### Basic Example

Place `#[serde_hash]` above your derive and mark fields with `#[serde(hash)]`. All standard serde attributes work alongside `hash`:

```rust
use serde::{Serialize, Deserialize};
use serde_hash::serde_hash;
use serde_hash::hashids::SerdeHashOptions;

#[serde_hash]
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
	#[serde(hash, alias = "identifier")]
	pub id: u64,
	#[serde(rename = "user_name")]
	pub name: String,
	pub age: u8,
}

fn main() {
	SerdeHashOptions::new()
		.with_salt("my-secret-salt")
		.with_min_length(10)
		.with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
		.build();

	let user = User {
		id: 158674,
		name: "Dan Smith".to_string(),
		age: 47,
	};

	let json_string = serde_json::to_string(&user).unwrap();
	println!("{:?} -> {}", user, json_string);
	// Output: User { id: 158674, ... } -> {"id":"qKknODM7Ej","user_name":"Dan Smith","age":47}
}
```

Notice how `#[serde(hash, alias = "identifier")]` works -- `hash` triggers hash encoding while `alias` is handled by serde normally. The `rename` on `name` also works as expected.

### Deserialization Example

Hash strings are transparently decoded back to numeric IDs:

```rust
use serde::{Serialize, Deserialize};
use serde_hash::serde_hash;
use serde_hash::hashids::SerdeHashOptions;

#[serde_hash]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
	#[serde(hash)]
	pub id: u64,
	pub name: String,
	pub age: u8,
}

fn main() {
	SerdeHashOptions::new()
		.with_salt("my-secret-salt")
		.with_min_length(10)
		.with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
		.build();

	let user = User { id: 158674, name: "Dan Smith".to_string(), age: 47 };
	let json = serde_json::to_string(&user).unwrap();

	let deserialized: User = serde_json::from_str(&json).unwrap();
	assert_eq!(user, deserialized);
}
```

### Using with Vectors and Options

`#[serde(hash)]` works with `Vec<T>`, `Option<T>`, and `Option<Vec<T>>` where `T` is an unsigned integer:

```rust
use serde::{Serialize, Deserialize};
use serde_hash::serde_hash;
use serde_hash::hashids::SerdeHashOptions;

#[serde_hash]
#[derive(Serialize, Deserialize, Debug)]
pub struct DataWithCollections {
	#[serde(hash)]
	pub id: u64,
	pub name: String,
	#[serde(hash)]
	pub values: Vec<u8>,
	#[serde(hash)]
	pub optional_id: Option<u32>,
	#[serde(hash)]
	pub optional_values: Option<Vec<u16>>,
}

fn main() {
	SerdeHashOptions::new()
		.with_salt("my-secret-salt")
		.build();

	let data = DataWithCollections {
		id: 158674,
		name: "Dan Smith".to_string(),
		values: vec![1, 2, 3, 4, 5],
		optional_id: Some(42),
		optional_values: Some(vec![10, 20, 30]),
	};

	let json = serde_json::to_string(&data).unwrap();
	println!("{}", json);

	let deserialized: DataWithCollections = serde_json::from_str(&json).unwrap();
	println!("{:?}", deserialized);
}
```

### Generating Secure Salt

For production use, generate a cryptographically secure random salt:

```rust
use serde_hash::{hashids::SerdeHashOptions, salt::generate_salt};

let salt = generate_salt();
SerdeHashOptions::new()
.with_salt( & salt)
.with_min_length(10)
.build();
```

### How It Works

The `#[serde_hash]` attribute macro runs **before** serde's derive macros. It transforms `#[serde(hash)]` into serde's `#[serde(with = "...")]` attribute, pointing to built-in serialize/deserialize functions that handle hash encoding. This means:

- Serde's own derive generates the `Serialize`/`Deserialize` implementations
- All serde attributes (`rename`, `alias`, `default`, `skip`, `flatten`, etc.) work normally
- Hash encoding is applied only to the marked fields via serde's `with` mechanism

## Why Use serde_hash?

- **Obfuscation**: Hide your internal database IDs from API consumers
- **Serde compatible**: Works with all serde attributes and any serde-based format (JSON, TOML, MessagePack, etc.)
- **Predictability**: Unlike UUID generation, the same ID always hashes to the same string with the same settings
- **Transparency**: Your application code works with numeric IDs while your API exposes hash strings
- **Simplicity**: Add one attribute macro and configure once

## License

[MIT License](LICENSE)

---

This project is built with Rust and uses the [hash-ids](https://crates.io/crates/hash-ids) crate for hash ID generation.

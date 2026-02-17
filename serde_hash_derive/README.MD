# serde_hash_derive

Proc-macro implementation for the [serde_hash](https://crates.io/crates/serde_hash) library. This crate is not meant to be used directly -- use `serde_hash` instead, which re-exports everything from this crate.

## Provided Macros

### `#[serde_hash]` (Recommended)

Attribute macro that enables `#[serde(hash)]` on struct fields. Place it above `#[derive(Serialize, Deserialize)]` to extend serde with hash encoding while preserving all standard serde attributes.

```rust
use serde::{Deserialize, Serialize};
use serde_hash::serde_hash;

#[serde_hash]
#[derive(Serialize, Deserialize)]
pub struct User {
	#[serde(hash, alias = "identifier")]
	pub id: u64,
	#[serde(rename = "user_name")]
	pub name: String,
}
```

### `#[derive(HashIds)]` (Legacy)

The original derive macro that generates complete `Serialize`/`Deserialize` implementations. Kept for backward compatibility but does not support serde attributes like `rename` or `alias`.

```rust
use serde_hash::HashIds;

#[derive(HashIds)]
pub struct User {
	#[hash]
	pub id: u64,
	pub name: String,
}
```

For new code, prefer `#[serde_hash]` with `#[derive(Serialize, Deserialize)]`.

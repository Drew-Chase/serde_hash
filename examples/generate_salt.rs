use serde::{Deserialize, Serialize};
use serde_hash::{hashids::SerdeHashOptions, salt::generate_salt, serde_hash};

// TestData struct using #[serde_hash] to enable hash ID serialization/deserialization
#[serde_hash]
#[derive(Serialize, Deserialize, Debug)]
pub struct TestData {
    // This field will be hashed in the JSON output
    #[serde(hash)]
    pub id: u64,
    // Regular fields that will be serialized normally
    pub name: String,
    pub age: u8,
}

fn main() {
    // Generate a cryptographically secure random salt string (32 alphanumeric characters)
    let salt = generate_salt();
    println!("Salt: {}", salt);

    // Configure the hash ID generation options using the builder pattern
    SerdeHashOptions::new()
        .with_salt(salt)
        .with_min_length(10)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // Create a test instance with sample data
    let data = TestData {
        id: 158674,
        name: "Dan Smith".to_string(),
        age: 47,
    };

    // Serialize the struct to JSON string
    let json_string = serde_json::to_string(&data).unwrap();

    // Display both the original struct and its JSON representation
    println!("{:?} -> {}", data, json_string);
}

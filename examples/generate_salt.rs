// Import required dependencies from the serde_hash_impl crate and the derive macro
use serde_hash::{hashids::SerdeHashOptions, salt::generate_salt, HashIds};

// TestData struct with HashIds derive macro to enable hash ID serialization/deserialization
// The #[hash] attribute marks fields that should be encoded/decoded as hash IDs
#[derive(HashIds, Debug)]
pub struct TestData {
    // This field will be hashed in the JSON output
    #[hash]
    pub id: u64,
    // Regular fields that will be serialized normally
    pub name: String,
    pub age: u8,
}

fn main() {
    // Generate a cryptographically secure random salt string (32 alphanumeric characters)
    let salt = generate_salt(); // this will generate a random string of characters.
    println!("Salt: {}", salt);

    // Configure the hash ID generation options using the builder pattern
    // These settings will be applied globally for all hash ID operations
    SerdeHashOptions::new()
        .with_salt(salt)                // Set the random salt for hash generation
        .with_min_length(10)            // Ensure hash IDs are at least 10 characters long
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890") // Set character set for hash encoding
        .build();

    // Create a test instance with sample data
    let data = TestData {
        id: 158674,              // This numeric ID will be hashed in the JSON output
        name: "Dan Smith".to_string(),
        age: 47,
    };

    // Serialize the struct to JSON string
    // The HashIds derive macro has implemented custom serialization 
    // that will hash the 'id' field in the resulting JSON
    let json_string = serde_json::to_string(&data).unwrap();

    // Display both the original struct and its JSON representation
    println!("{:?} -> {}", data, json_string);
}
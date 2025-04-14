// Import the SerdeHashOptions struct from serde_hash_impl crate for configuring hash ID generation
use serde_hash::hashids::SerdeHashOptions;
use serde_hash::HashIds;

// TestData structure with one field marked for hashing
#[derive(HashIds, Debug)]
pub struct TestData {
    // The #[hash] attribute marks this field to be encoded/decoded during serialization
    #[hash]
    pub id: u64,
    // Regular fields that will be serialized normally
    pub name: String,
    pub age: u8,
}

fn main() {
    // Configure the hash ID generation settings
    SerdeHashOptions::new()
        // Set the salt for a more secure and unique hash generation
        .with_salt("hello world")
        // Ensure generated hashes are at least 10 characters long
        .with_min_length(10)
        // Define the character set used for generating hash strings
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // Create a test data instance
    let data = TestData {
        id: 158674,         // This value will be hashed during serialization
        name: "Dan Smith".to_string(),
        age: 47,
    };

    // Serialize the data structure to a JSON string
    // The HashIds derive macro ensures the id field gets hashed
    let json_string = serde_json::to_string(&data).unwrap();

    // Deserialize the JSON string back to a TestData struct
    // The HashIds derive macro ensures the hashed id gets properly decoded
    let deserialized: TestData = serde_json::from_str(&json_string).unwrap();

    // Print the original data, serialized JSON, and deserialized data
    println!("{:?} -> {} -> {:?}", data, json_string, deserialized);
}
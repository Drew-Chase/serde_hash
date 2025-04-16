use serde_hash::hashids::SerdeHashOptions;
use serde_hash::HashIds;

// TestData structure with HashIds derives macro for automated hash encoding/decoding
// The structure includes a hashable ID field and regular non-hashed fields
#[derive(HashIds, Debug)]
pub struct TestData {
    // Mark this field to be hashed during serialization and deserialization
    #[hash]
    pub id: u64,
    // Regular string field that won't be hashed
    pub name: String,
    // Regular numeric field that won't be hashed
    #[hash]
    pub age: Option<u8>,
}

fn main() {
    // Configure the hash ID generator with custom settings
    // This affects how numeric IDs are converted to hash strings
    // All of these fields are optional.
    SerdeHashOptions::new()
        // Set the cryptographic salt for the hash algorithm
        .with_salt("hello world")
        // Ensure generated hashes are at least 10 characters long
        .with_min_length(10)
        // Define the character set used in the hash encoding (alphanumeric in this case)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // Create a test data instance with sample values
    let data = TestData {
        id: 158674,
        name: "Dan Smith".to_string(),
        age: Some(47),
    };

    // Serialize the data to JSON string
    // The id field will be automatically hashed based on our HashIds implementation
    let json_string = serde_json::to_string(&data).unwrap();

    // Print both the original data object and its JSON representation
    println!("{:?} -> {}", data, json_string);
}
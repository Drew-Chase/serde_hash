use serde::{Deserialize, Serialize};
use serde_hash::hashids::SerdeHashOptions;
use serde_hash::serde_hash;

// TestData structure with one field marked for hashing
#[serde_hash]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TestData {
    // The #[serde(hash)] attribute marks this field to be encoded/decoded during serialization
    #[serde(hash)]
    pub id: u64,
    // Regular fields that will be serialized normally
    pub name: String,
    pub age: u8,
}

fn main() {
    // Configure the hash ID generation settings
    SerdeHashOptions::new()
        .with_salt("hello world")
        .with_min_length(10)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // Create a test data instance
    let data = TestData {
        id: 158674,
        name: "Dan Smith".to_string(),
        age: 47,
    };

    // Serialize the data structure to a JSON string
    let json_string = serde_json::to_string(&data).unwrap();

    // Deserialize the JSON string back to a TestData struct
    let deserialized: TestData = serde_json::from_str(&json_string).unwrap();

    // Print the original data, serialized JSON, and deserialized data
    println!("{:?} -> {} -> {:?}", data, json_string, deserialized);
}

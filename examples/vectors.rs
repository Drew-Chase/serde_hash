use serde::{Deserialize, Serialize};
use serde_hash::hashids::SerdeHashOptions;
use serde_hash::serde_hash;

// TestData structure demonstrating all supported hash field types
#[serde_hash]
#[derive(Serialize, Deserialize, Debug)]
pub struct TestData {
    // Plain numeric field
    #[serde(hash)]
    pub id: u64,
    // Regular string field that won't be hashed
    pub name: String,
    // Vec of numeric type
    #[serde(hash)]
    pub vec: Vec<u8>,
    // Optional Vec of numeric type
    #[serde(hash)]
    pub optional_vec: Option<Vec<u16>>,
    // Optional numeric type
    #[serde(hash)]
    pub age: Option<usize>,
}

fn main() {
    // Configure the hash ID generator with custom settings
    SerdeHashOptions::new()
        .with_salt("hello world")
        .with_min_length(10)
        .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .build();

    // Create a test data instance with sample values
    let data = TestData {
        id: 158674,
        name: "Dan Smith".to_string(),
        age: Some(47),
        vec: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        optional_vec: Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
    };

    // Serialize the data to JSON string
    let json_string = serde_json::to_string(&data).unwrap();

    let deserialized: TestData = serde_json::from_str(&json_string).unwrap();

    // Print both the original data object and its JSON representation
    println!("{:?}\n-> {}\n-> {:?}", data, json_string, deserialized);
}

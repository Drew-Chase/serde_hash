mod test_serialization {
    use serde_hash::hashids::SerdeHashOptions;
    use serde_hash_derive::HashIds;

    #[derive(HashIds, Debug, PartialEq)]
    pub struct TestData {
        #[hash]
        pub id: u64,
        pub name: String,
        pub age: u8,
    }
    #[test]
    fn test_basic() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();
        let data = TestData {
            id: 158674,
            name: "Dan Smith".to_string(),
            age: 47,
        };

        let json_string = serde_json::to_string(&data).unwrap();
        // {"id":"qKknODM7Ej","name":"Dan Smith","age":47}
        assert!(json_string.contains("qKknODM7Ej"));
    }

    #[test]
    fn test_deserialization() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();

        // Create a test data instance
        let data = TestData {
            id: 158674, // This value will be hashed during serialization
            name: "Dan Smith".to_string(),
            age: 47,
        };

        let json_string = serde_json::to_string(&data).unwrap();
        let deserialized: TestData = serde_json::from_str(&json_string).unwrap();
        assert_eq!(deserialized, data)
    }
}

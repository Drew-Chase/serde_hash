mod test_serialization {
    use serde::{Deserialize, Serialize};
    use serde_hash::hashids::SerdeHashOptions;
    use serde_hash::serde_hash;

    #[serde_hash]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct TestData {
        #[serde(hash)]
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
        assert!(json_string.contains("qKknODM7Ej"));
    }

    #[test]
    fn test_deserialization() {
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
        let deserialized: TestData = serde_json::from_str(&json_string).unwrap();
        assert_eq!(deserialized, data);
    }

    // Test that serde attributes (rename, alias) work alongside hash
    #[serde_hash]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct TestDataWithSerde {
        #[serde(hash, alias = "identifier")]
        pub id: u64,
        #[serde(rename = "test_name")]
        pub name: String,
    }

    #[test]
    fn test_serde_rename_with_hash() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();
        let data = TestDataWithSerde {
            id: 100,
            name: "Alice".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        // name should be serialized as "test_name"
        assert!(json.contains("test_name"));
        assert!(!json.contains("\"name\""));
        // id should be hashed (not a plain number)
        assert!(!json.contains("\"id\":100"));
    }

    #[test]
    fn test_alias_deserialization() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();
        let data = TestDataWithSerde {
            id: 100,
            name: "Alice".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        // Replace "id" key with "identifier" alias and verify deserialization works
        let with_alias = json.replace("\"id\"", "\"identifier\"");
        let deserialized: TestDataWithSerde = serde_json::from_str(&with_alias).unwrap();
        assert_eq!(deserialized.id, 100);
    }

    // Test Option hash field
    #[serde_hash]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct TestDataWithOption {
        #[serde(hash)]
        pub id: Option<u64>,
    }

    #[test]
    fn test_option_some() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();
        let data = TestDataWithOption { id: Some(42) };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TestDataWithOption = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_option_none() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();
        let data = TestDataWithOption { id: None };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("null"));
        let deserialized: TestDataWithOption = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    // Test Vec hash field
    #[serde_hash]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct TestDataWithVec {
        #[serde(hash)]
        pub ids: Vec<u64>,
    }

    #[test]
    fn test_vec_hash() {
        SerdeHashOptions::new()
            .with_salt("hello world")
            .with_min_length(10)
            .with_alphabet("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
            .build();
        let data = TestDataWithVec {
            ids: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TestDataWithVec = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ids, vec![1, 2, 3]);
    }
}

use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_hash_impl::hashids::{decode_single, encode_single, SerdeHashOptions};

#[derive(Debug)]
pub struct TestData {
    pub id: u64,
    pub name: String,
    pub age: u8,
    pub vec: Vec<u8>,
}

fn main() {
    SerdeHashOptions::new().with_salt("hello world").build();
    let data = TestData {
        id: 158674,
        name: "Dan Smith".to_string(),
        age: 47,
        vec: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };

    let json_string = serde_json::to_string(&data).unwrap();
    let deserialized: TestData = serde_json::from_str(&json_string).unwrap();
    println!("{:?} -> {} -> {:?}", data, json_string, deserialized);
}

impl Serialize for TestData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("id", &encode_single(self.id))?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("age", &self.age)?;
        let mut vec_str = vec![];
        for v in self.vec.iter() {
            vec_str.push(encode_single(*v as u64));
        }
        map.serialize_entry("vec", &vec_str)?;

        map.end()
    }
}

impl<'de> Deserialize<'de> for TestData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        enum Field {
            #[serde(rename = "id")]
            Id,
            #[serde(rename = "name")]
            Name,
            #[serde(rename = "age")]
            Age,
            #[serde(rename = "vec")]
            Vec,
        }

        struct TestDataVisitor;

        impl<'de> Visitor<'de> for TestDataVisitor {
            type Value = TestData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct TestData")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id: Option<u64> = None;
                let mut name: Option<String> = None;
                let mut age: Option<u8> = None;
                let mut vec: Option<Vec<u8>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            } else if let Ok(string_value) = map.next_value::<String>() {
                                if let Ok(u64_value) = decode_single(string_value) {
                                    id = Some(u64_value);
                                }
                            }
                        }
                        Field::Vec => {
                            if vec.is_some() {
                                return Err(serde::de::Error::duplicate_field("vec"));
                            } else if let Ok(vec_string) = map.next_value::<Vec<String>>() {
                                let mut tmp:Vec<u8> = vec![];
                                for v in vec_string {
                                    if let Ok(u64_value) = decode_single(v) {
                                        tmp.push(u64_value as u8);
                                    }
                                }
                                vec = Some(tmp);
                            }
                        }
                        Field::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            } else if let Ok(string_value) = map.next_value::<String>() {
                                name = Some(string_value);
                            }
                        }
                        Field::Age => {
                            if age.is_some() {
                                return Err(serde::de::Error::duplicate_field("age"));
                            } else if let Ok(u8_value) = map.next_value::<u8>() {
                                age = Some(u8_value);
                            }
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
                let age = age.ok_or_else(|| serde::de::Error::missing_field("age"))?;
                let vec = vec.ok_or_else(|| serde::de::Error::missing_field("vec"))?;

                Ok(TestData { id, name, age, vec })
            }
        }
        deserializer.deserialize_struct("TestData", &["id", "name", "age", "vec"], TestDataVisitor)
    }
}

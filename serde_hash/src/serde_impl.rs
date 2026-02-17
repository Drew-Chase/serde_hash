use crate::hashids::{decode_single, encode_single};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Trait for numeric types that can be hash-encoded as u64.
pub trait HashNumeric: Copy {
    fn to_u64(self) -> u64;
    fn from_u64(v: u64) -> Self;
}

macro_rules! impl_hash_numeric {
    ($($t:ty),*) => {
        $(
            impl HashNumeric for $t {
                fn to_u64(self) -> u64 { self as u64 }
                fn from_u64(v: u64) -> Self { v as Self }
            }
        )*
    }
}

impl_hash_numeric!(u8, u16, u32, u64, u128, usize);

/// Serde `with` module for plain numeric fields (`u8`, `u16`, `u32`, `u64`, `u128`, `usize`).
///
/// Usage: `#[serde(with = "serde_hash::serde_impl::numeric")]`
pub mod numeric {
    use super::*;

    pub fn serialize<T: HashNumeric, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let encoded = encode_single(value.to_u64());
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, T: HashNumeric, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        let s = String::deserialize(deserializer)?;
        let decoded = decode_single(&s).map_err(serde::de::Error::custom)?;
        Ok(T::from_u64(decoded))
    }
}

/// Serde `with` module for `Vec<T>` where `T` is a numeric type.
///
/// Usage: `#[serde(with = "serde_hash::serde_impl::vec_numeric")]`
pub mod vec_numeric {
    use super::*;

    pub fn serialize<T: HashNumeric, S: Serializer>(
        value: &[T],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let encoded: Vec<String> = value.iter().map(|v| encode_single(v.to_u64())).collect();
        encoded.serialize(serializer)
    }

    pub fn deserialize<'de, T: HashNumeric, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<T>, D::Error> {
        let strings = Vec::<String>::deserialize(deserializer)?;
        strings
            .into_iter()
            .map(|s| {
                let decoded = decode_single(&s).map_err(serde::de::Error::custom)?;
                Ok(T::from_u64(decoded))
            })
            .collect()
    }
}

/// Serde `with` module for `Option<T>` where `T` is a numeric type.
///
/// Usage: `#[serde(with = "serde_hash::serde_impl::option_numeric")]`
pub mod option_numeric {
    use super::*;

    pub fn serialize<T: HashNumeric, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(v) => {
                let encoded = encode_single(v.to_u64());
                serializer.serialize_some(&encoded)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, T: HashNumeric, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let decoded = decode_single(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(T::from_u64(decoded)))
            }
            None => Ok(None),
        }
    }
}

/// Serde `with` module for `Option<Vec<T>>` where `T` is a numeric type.
///
/// Usage: `#[serde(with = "serde_hash::serde_impl::option_vec_numeric")]`
pub mod option_vec_numeric {
    use super::*;

    pub fn serialize<T: HashNumeric, S: Serializer>(
        value: &Option<Vec<T>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(vec) => {
                let encoded: Vec<String> =
                    vec.iter().map(|v| encode_single(v.to_u64())).collect();
                serializer.serialize_some(&encoded)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, T: HashNumeric, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<T>>, D::Error> {
        let opt = Option::<Vec<String>>::deserialize(deserializer)?;
        match opt {
            Some(strings) => {
                let result: Result<Vec<T>, _> = strings
                    .into_iter()
                    .map(|s| {
                        let decoded = decode_single(&s).map_err(serde::de::Error::custom)?;
                        Ok(T::from_u64(decoded))
                    })
                    .collect();
                Ok(Some(result?))
            }
            None => Ok(None),
        }
    }
}

use crate::salt::generate_salt;
use anyhow::Result;
use hash_ids::HashIds;
use log::debug;

/// Decodes a given hash string into a vector of `u64` integers.
///
/// # Arguments
///
/// * `hash` - A string slice that holds the hash to be decoded.
///
/// # Returns
///
/// A vector of `u64` integers that were encoded in the given hash string.
pub fn decode(hash: impl AsRef<str>) -> Result<Vec<u64>> {
    let hash = hash.as_ref();
    let hash_ids = hashids();
    let decode = hash_ids.decode(hash)?;
    debug!("Decoding: {} -> {:?}", hash, decode);
    Ok(decode)
}

/// Encodes a slice of `u64` integers into a hash string.
///
/// # Arguments
///
/// * `data` - A slice of `u64` integers to be encoded.
///
/// # Returns
///
/// A string that represents the encoded hash of the input data.
pub fn encode(data: &[u64]) -> String {
    let hash_ids = hashids();
    let encode = hash_ids.encode(data);
    debug!("Encoding: {:?} -> {}", data, encode);
    encode
}

/// Decodes a hash string into a single `u64` value.
///
/// # Arguments
///
/// * `hash` - A string reference that contains the hash to be decoded.
///
/// # Returns
///
/// * On success, returns a single `u64` value that was encoded in the hash.
/// * On failure, returns an error if the hash does not decode to exactly one `u64` value,
///   or if an error occurs during decoding.
pub fn decode_single(hash: impl AsRef<str>) -> Result<u64> {
    let hash = hash.as_ref(); // Extracts the underlying string reference from the wrapper.
    let decode = decode(hash)?; // Attempts to decode the hash into a vector of `u64` integers.

    // Check if the decoded result contains exactly one value.
    if decode.len() != 1 {
        return Err(anyhow::Error::msg(format!("Invalid hash: {}", hash))); // Returns an error if not.
    }

    // Successfully return the single decoded value.
    Ok(decode[0])
}

/// Encodes a single `u64` value into a hash string.
///
/// # Arguments
///
/// * `data` - A single `u64` value to be encoded into a hash.
///
/// # Returns
///
/// * A string that represents the encoded hash of the input value.
pub fn encode_single(data: u64) -> String {
    encode(&[data]) // Calls the `encode` function with the input value wrapped in a slice.
}

fn hashids() -> HashIds {
    let options = get_hash_options();
    HashIds::builder()
        .with_salt(options.salt.as_str())
        .with_min_length(options.min_length)
        .with_alphabet(options.alphabet.as_str())
        .finish()
        .unwrap()
}

use std::sync::OnceLock;

/// Configuration options for the hash ID generation.
///
/// This struct stores the configuration parameters used by the hash ID generator,
/// including the salt for randomization, minimum length of generated hashes,
/// and the alphabet used for encoding.
pub struct SerdeHashOptions {
    /// Salt string used to randomize hash generation
    pub salt: String,
    /// Minimum length of generated hash strings
    pub min_length: usize,
    /// Character set used for encoding values into hash strings
    pub alphabet: String,
}

impl Default for SerdeHashOptions {
    /// Creates default configuration with:
    /// - A randomly generated salt
    /// - Minimum hash length of 8 characters
    /// - Standard alphanumeric alphabet (a-z, A-Z, 0-9)
    fn default() -> Self {
        Self {
            salt: generate_salt(), // Generate a random salt string
            min_length: 8,         // Set default minimum hash length
            alphabet: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".to_string(),
        }
    }
}

/// Global singleton instance of hash options initialized lazily
static HASH_OPTIONS: OnceLock<SerdeHashOptions> = OnceLock::new();

/// Provides access to the global hash configuration options.
///
/// This function returns a reference to the global hash configuration.
/// If the configuration hasn't been initialized yet, it will initialize
/// it with default values.
///
/// # Returns
///
/// A static reference to the global `SerdeHashOptions` instance
pub fn get_hash_options() -> &'static SerdeHashOptions {
    HASH_OPTIONS.get_or_init(SerdeHashOptions::default)
}

impl SerdeHashOptions {
    /// Creates a new `SerdeHashOptions` instance with default values.
    ///
    /// # Returns
    ///
    /// A new instance of `SerdeHashOptions` with default salt (randomly generated),
    /// minimum length of 8, and standard alphanumeric alphabet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom salt for the hash options.
    ///
    /// # Arguments
    ///
    /// * `salt` - The salt string to use for hashing can be any type that can be
    ///   converted to a string reference.
    ///
    /// # Returns
    ///
    /// Self with the updated salt value for method chaining.
    pub fn with_salt(mut self, salt: impl AsRef<str>) -> Self {
        self.salt = salt.as_ref().to_string(); // Convert input to a String and store it
        self
    }

    /// Sets a custom minimum length for generated hash IDs.
    ///
    /// # Arguments
    ///
    /// * `min_length` - The minimum length for generated hash IDs must be convertible to usize.
    ///
    /// # Returns
    ///
    /// Self with the updated minimum length for method chaining.
    ///
    /// # Panics
    ///
    /// Panics if the provided value cannot be converted to an usize.
    pub fn with_min_length<T>(mut self, min_length: T) -> Self
    where
        T: TryInto<usize>,
        <T as TryInto<usize>>::Error: std::fmt::Debug,
    {
        self.min_length = min_length.try_into().expect("Failed to convert to usize");
        self
    }

    /// Sets a custom alphabet for generating hash IDs.
    ///
    /// # Arguments
    ///
    /// * `alphabet` - The custom alphabet to use can be any type that can be
    ///   converted to a string reference.
    ///
    /// # Returns
    ///
    /// Self with the updated alphabet for method chaining.
    pub fn with_alphabet(mut self, alphabet: impl AsRef<str>) -> Self {
        self.alphabet = alphabet.as_ref().to_string(); // Convert input to a String and store it
        self
    }

    /// Finalizes the configuration and stores it in the global `HASH_OPTIONS`.
    ///
    /// This method sets the configured options as the global hash options that
    /// will be used for all subsequent hash operations in the application.
    /// Once set, the options cannot be changed as they're stored in a `OnceLock`.
    pub fn build(self) {
        let _ = HASH_OPTIONS.set(self); // Store the configured options in the global OnceLock
    }
}

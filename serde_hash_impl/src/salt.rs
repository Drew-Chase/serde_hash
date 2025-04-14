use log::debug;
use rand::distr::Alphanumeric;
use rand::Rng;

/// Generates a cryptographically secure random salt string of 32 characters.
///
/// This function creates a salt string composed of alphanumeric characters
/// (a-z, A-Z, 0-9) for use in hashing operations. The salt helps ensure
/// that hash outputs are unique even for identical inputs.
///
/// # Returns
///
/// A randomly generated String of 32 alphanumeric characters.
///
/// # Example
///
/// ```
/// let salt = serde_hash::salt::generate_salt();
/// // Returns a random string like "a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6"
/// ```
pub fn generate_salt() -> String {
    debug!("Generating salt"); // Log when salt generation begins
    rand::rng()
        .sample_iter(&Alphanumeric) // Generate a stream of random alphanumeric chars
        .take(32)                   // Limit to 32 characters
        .map(char::from)            // Convert raw values to characters
        .collect()                  // Collect characters into a String
}
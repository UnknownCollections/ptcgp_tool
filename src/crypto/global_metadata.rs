use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};

// Define a type alias for AES-128 in CTR mode with a 128-bit big-endian counter.
type Aes128Ctr = ctr::Ctr128BE<Aes128>;

/// Decrypts data that has been encrypted with a custom AES-CTR implementation.
///
/// # Parameters
/// - `data`: The input byte slice containing a 4-byte little-endian header (which indicates the ciphertext length)
///   followed by the ciphertext itself.
/// - `key`: A 16-byte encryption key.
/// - `key_xor`: A 64-bit value used to XOR with the original key for an extra layer of key obfuscation.
///
/// # Returns
/// A vector containing the decrypted plaintext.
///
/// # Explanation
/// 1. **Key Obfuscation:**  
///    The provided `key` is XORed with the little-endian byte representation of `key_xor`.
///    The 8-byte representation of `key_xor` is cycled over the 16-byte key. This extra step
///    hides the actual key by combining it with an additional value.
///
/// 2. **Data Parsing:**  
///    The input `data` starts with a 4-byte header that indicates the length of the ciphertext.
///    The function separates these 4 bytes and converts them into a `u32` to verify the data integrity.
///
/// 3. **IV Setup for CTR Mode:**  
///    The custom encryption scheme uses an initial counter of 1 (instead of 0) for the first block.
///    To achieve this, an Initialization Vector (IV) is constructed with its lower 8 bytes set to the big-endian
///    representation of `1` and the remaining bytes set to zero.
///
/// 4. **Decryption:**  
///    Using the adjusted key and the IV, the AES-CTR cipher is initialized. The ciphertext is then decrypted
///    in place by applying the keystream.
///
/// Note: This function assumes that the input data is well-formed and will panic if the header or lengths are incorrect.
pub fn decrypt_data(data: &[u8], key: [u8; 16], key_xor: u64) -> Vec<u8> {
    // Convert the key_xor value to its little-endian byte representation.
    let key_xor_bytes = key_xor.to_le_bytes();

    // XOR each byte of the key with the corresponding byte from the key_xor_bytes.
    // The key_xor_bytes (8 bytes) is repeated (cycled) to match the 16-byte key length.
    let key_bytes: [u8; 16] = key
        .iter()
        .zip(key_xor_bytes.iter().cycle())
        .map(|(&b, &k)| b ^ k)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();

    // The first 4 bytes of `data` represent the ciphertext length (stored in little-endian order).
    let (data_len_bytes, ciphertext) = data.split_at(4);
    let data_len = u32::from_le_bytes(data_len_bytes.try_into().unwrap());
    debug_assert_eq!(data_len as usize, ciphertext.len());

    // Prepare the Initialization Vector (IV) for AES-CTR.
    // The custom implementation increments the counter before encryption,
    // so the first block is encrypted with counter = 1. Here, we construct a 16-byte IV
    // where the lower 8 bytes (big-endian) represent the number 1 and the rest are zero.
    let mut iv = [0u8; 16];
    iv[8..16].copy_from_slice(&1u64.to_be_bytes());

    // Initialize the AES-CTR cipher with the obfuscated key and custom IV.
    let mut cipher = Aes128Ctr::new(&key_bytes.into(), &iv.into());

    // Create a mutable copy of the ciphertext and decrypt it in place using the keystream.
    let mut decrypted = ciphertext.to_vec();
    cipher.apply_keystream(&mut decrypted);
    decrypted
}

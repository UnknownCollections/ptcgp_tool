use std::hash::Hasher;

/// Computes a non–cryptographic hash by accumulating the input bytes with a fixed multiplier
/// using wrapping arithmetic.
///
/// This function processes data in 4–byte chunks (or byte–by–byte when special cases apply) and
/// combines each segment with the running hash value. It is optimized for speed and decent bit
/// dispersion rather than cryptographic security.
///
/// # Arguments
///
/// * `hash` - The starting hash value.
/// * `data` - The input byte slice to be hashed.
/// * `multiplier` - The constant multiplier used to mix in each data segment.
///
/// # Returns
///
/// Returns the updated hash value after processing all input data.
///
/// # Details
///
/// - For exactly 4 bytes of input, each byte is processed individually to mix into the hash.
/// - For longer inputs, the data is divided into 4–byte little–endian words. Each word is mixed
///   into the hash state using the multiplier.
/// - After processing full 4–byte blocks, any remaining bytes (“tail”) are re–processed to improve
///   the avalanche effect.
pub fn pocket_hash_accumulate(mut hash: u64, data: &[u8], multiplier: u64) -> u64 {
    let len = data.len();

    // Special-case: process each byte separately when data length is exactly 4.
    if len == 4 {
        for &byte in data {
            // Use wrapping arithmetic to prevent overflow panics.
            hash = (byte as u64).wrapping_add(multiplier.wrapping_mul(hash));
        }
        return hash;
    }

    // Process the bulk of the input in 4-byte chunks.
    let mut offset = 0;
    while offset < len.saturating_sub(4) {
        // Convert the 4-byte slice into a little-endian u32, then cast to u64.
        let word = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as u64;
        hash = word.wrapping_add(multiplier.wrapping_mul(hash));
        offset += 4;
    }

    // Adjust the offset to re–process the tail bytes.
    if len >= 5 {
        offset = ((len - 5) / 4) * 4 + 4;
    }

    // Process any remaining bytes individually.
    for &byte in &data[offset..] {
        hash = (byte as u64).wrapping_add(multiplier.wrapping_mul(hash));
    }

    hash
}

/// A non–cryptographic code hasher that accumulates a hash using a SHA1–style algorithm.
///
/// This hasher processes input data using a fixed multiplier to mix bytes into an internal state.
/// It provides a simple interface to compute the hash of data in one call.
pub struct Il2CppPocketCodeHasher {
    /// The current internal hash state.
    hash: u64,
    /// The multiplier used for mixing new data into the hash.
    multiplier: u64,
}

impl Il2CppPocketCodeHasher {
    /// Creates a new `Il2CppPocketCodeHasher` with an initial hash state of zero.
    ///
    /// # Arguments
    ///
    /// * `multiplier` - The constant multiplier to mix data into the hash.
    pub fn new(multiplier: u64) -> Self {
        Self {
            hash: 0,
            multiplier,
        }
    }

    /// Computes the hash for the provided data in one call.
    ///
    /// # Arguments
    ///
    /// * `data` - The input byte slice to hash.
    /// * `multiplier` - The multiplier used in the hash computation.
    ///
    /// # Returns
    ///
    /// Returns the computed hash as a `u64`.
    pub fn hash(data: &[u8], multiplier: u64) -> u64 {
        let mut hasher = Self::new(multiplier);
        hasher.write(data);
        hasher.finish()
    }
}

impl Hasher for Il2CppPocketCodeHasher {
    /// Finalizes the hasher and returns the computed hash as a `u64`.
    ///
    /// This method casts the internal `u64` hash state to `u64`.
    fn finish(&self) -> u64 {
        self.hash
    }

    /// Updates the hash state by processing the provided byte slice.
    ///
    /// This method uses `pocket_hash_accumulate` to mix in the new data.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The input byte slice to incorporate into the hash.
    fn write(&mut self, bytes: &[u8]) {
        self.hash = pocket_hash_accumulate(self.hash, bytes, self.multiplier);
    }
}

/// A hasher that computes an XOR–based checksum over the input data.
///
/// This hasher processes data in 16–byte blocks, utilizing SSE2 SIMD instructions for acceleration
/// when available, and falls back to scalar operations otherwise. The final result is an 8–bit
/// checksum zero–extended to a 64–bit value.
pub struct Il2CppXorCodeHasher {
    /// Indicates if the CPU supports SSE2 instructions for SIMD acceleration.
    sse2_available: bool,

    /// A 16–byte accumulator that holds the XOR result of processed blocks.
    accum: [u8; 16],

    /// Buffer for storing bytes that do not complete a full 16–byte block.
    leftover: [u8; 16],
    /// The number of bytes currently stored in the `leftover` buffer.
    leftover_len: usize,
}

impl Il2CppXorCodeHasher {
    /// Constructs a new `Il2CppXorCodeHasher` and detects SSE2 availability at runtime.
    #[inline]
    pub fn new() -> Self {
        let sse2_available = is_x86_feature_detected!("sse2");

        Self {
            sse2_available,
            accum: [0; 16],
            leftover: [0; 16],
            leftover_len: 0,
        }
    }

    /// Computes the XOR checksum for the given data in a single call.
    ///
    /// # Arguments
    ///
    /// * `data` - The input byte slice to hash.
    ///
    /// # Returns
    ///
    /// Returns the computed checksum as a `u64`, with the checksum byte zero–extended.
    pub fn hash(data: &[u8]) -> u64 {
        let mut hasher = Self::new();
        hasher.write(data);
        hasher.finish()
    }

    /// XORs a 16–byte block into the accumulator.
    ///
    /// If SSE2 is available, this method leverages SIMD acceleration; otherwise, it performs a
    /// scalar byte–by–byte XOR.
    #[inline(always)]
    fn xor_16_bytes(&mut self, block: &[u8; 16]) {
        if self.sse2_available {
            unsafe { self.xor_16_bytes_sse2(block) }
        } else {
            for (a, &b) in self.accum.iter_mut().zip(block) {
                *a ^= b;
            }
        }
    }

    /// Performs a SIMD–accelerated XOR of a 16–byte block into the accumulator using SSE2 instructions.
    ///
    /// # Safety
    ///
    /// This function employs raw pointer operations and SSE2 intrinsics; callers must ensure that
    /// the CPU supports SSE2.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to a 16–byte array to be XORed into the accumulator.
    #[target_feature(enable = "sse2")]
    unsafe fn xor_16_bytes_sse2(&mut self, block: &[u8; 16]) {
        use core::arch::x86_64::*;
        unsafe {
            // Load the current accumulator and the new block into 128–bit SIMD registers.
            let reg_accum = _mm_loadu_si128(self.accum.as_ptr() as *const __m128i);
            let reg_block = _mm_loadu_si128(block.as_ptr() as *const __m128i);

            // Perform SIMD XOR on the registers.
            let result = _mm_xor_si128(reg_accum, reg_block);
            // Store the result back into the accumulator.
            _mm_storeu_si128(self.accum.as_mut_ptr() as *mut __m128i, result);
        }
    }
}

impl Default for Il2CppXorCodeHasher {
    /// Returns a default instance of `Il2CppXorCodeHasher` by invoking `new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Il2CppXorCodeHasher {
    /// Finalizes the hasher and returns the computed checksum.
    ///
    /// This method merges any buffered leftover bytes into the accumulator, reduces the 16–byte
    /// accumulator to a single checksum byte by XORing all bytes, and then zero–extends the result
    /// to a 64–bit value.
    #[inline]
    fn finish(&self) -> u64 {
        // Create a copy of the accumulator to avoid mutating self.
        let mut accum = self.accum;

        // Merge any leftover bytes into the accumulator.
        for (i, leftover) in self.leftover.iter().enumerate() {
            accum[i] ^= leftover;
        }

        // Reduce the 16–byte accumulator to a single checksum byte.
        let mut x = 0u8;
        for &b in &accum {
            x ^= b;
        }

        // Return the 8–bit checksum as a u64.
        x as u64
    }

    /// Updates the hasher state with new input data.
    ///
    /// The update process involves:
    /// 1. Completing any previously partial 16–byte block stored in `leftover`.
    /// 2. Processing full 16–byte blocks directly from the input.
    /// 3. Buffering any remaining bytes that do not complete a full block.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The input byte slice to stream into the hasher.
    fn write(&mut self, mut bytes: &[u8]) {
        // Complete the partial block if there are leftover bytes.
        if self.leftover_len > 0 {
            let needed = 16 - self.leftover_len;
            if bytes.len() < needed {
                // Not enough bytes to complete the block; append to leftover.
                self.leftover[self.leftover_len..(self.leftover_len + bytes.len())]
                    .copy_from_slice(bytes);
                self.leftover_len += bytes.len();
                return;
            } else {
                // Fill the leftover to complete a full 16–byte block.
                self.leftover[self.leftover_len..16].copy_from_slice(&bytes[..needed]);
                bytes = &bytes[needed..];

                // Process the completed block.
                let block = unsafe { &*(self.leftover.as_ptr() as *const [u8; 16]) };
                self.xor_16_bytes(block);

                // Reset the leftover buffer.
                self.leftover_len = 0;
            }
        }

        // Process full 16–byte blocks directly from the input.
        while bytes.len() >= 16 {
            let block = unsafe { &*(bytes.as_ptr() as *const [u8; 16]) };
            self.xor_16_bytes(block);
            bytes = &bytes[16..];
        }

        // Buffer any remaining bytes for future processing.
        self.leftover[..bytes.len()].copy_from_slice(bytes);
        self.leftover_len = bytes.len();
    }
}

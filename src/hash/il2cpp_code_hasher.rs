use std::hash::Hasher;

/// Computes a hash accumulation using a fixed multiplier and wrapping arithmetic.
///
/// The algorithm processes the input data in 4-byte chunks (or single bytes in special cases)
/// and mixes each word into the hash state using a fixed constant multiplier.
/// This non–cryptographic mixing function is designed for speed and decent dispersion.
pub fn pocket_hash_accumulate(mut hash: i64, data: &[u8]) -> i64 {
    // A fixed multiplier chosen to mix the current hash value with new data.
    const MULTIPLIER: i64 = 0x0002_317D_AE56_CBA9;
    let len = data.len();

    // Special-case: for exactly 4 bytes, process each byte individually.
    if len == 4 {
        for &byte in data {
            // Use wrapping arithmetic to allow overflow without panicking.
            hash = (byte as i64).wrapping_add(MULTIPLIER.wrapping_mul(hash));
        }
        return hash;
    }

    // Process input in 4-byte chunks.
    // Each 4-byte segment is interpreted as a little-endian u32 for consistent platform behavior.
    let mut offset = 0;
    while offset < len.saturating_sub(4) {
        let word = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as i64;
        hash = word.wrapping_add(MULTIPLIER.wrapping_mul(hash));
        offset += 4;
    }

    // Adjust the offset to re–process the trailing bytes.
    // This ensures that the tail of the data (which might have been part of a nearly complete block)
    // is mixed in a second pass to improve the hash’s avalanche properties.
    if len >= 5 {
        offset = ((len - 5) / 4) * 4 + 4;
    }

    // Process any remaining bytes (the “tail” of the input) individually.
    for &byte in &data[offset..] {
        hash = (byte as i64).wrapping_add(MULTIPLIER.wrapping_mul(hash));
    }

    hash
}

/// A hasher that accumulates a SHA1–style code hash.
pub struct Il2CppPocketCodeHasher {
    hash: i64,
}

impl Il2CppPocketCodeHasher {
    /// Creates a new hasher with an initial hash state of zero.
    pub fn new() -> Self {
        Self { hash: 0 }
    }

    /// Convenience function to compute a hash from the provided data in one call.
    pub fn hash(data: &[u8]) -> u64 {
        let mut hasher = Self::new();
        hasher.write(data);
        hasher.finish()
    }
}

impl Hasher for Il2CppPocketCodeHasher {
    /// Finalizes and returns the hash as an u64.
    fn finish(&self) -> u64 {
        self.hash as u64
    }

    /// Updates the hash state by accumulating the provided bytes.
    fn write(&mut self, bytes: &[u8]) {
        self.hash = pocket_hash_accumulate(self.hash, bytes);
    }
}

/// Hasher that computes a single-byte XOR checksum for all written bytes.
/// It processes data in 16-byte chunks, using 128-bit SSE2 instructions when available for speed.
/// The final checksum is an 8-bit value zero–extended to 64 bits.
pub struct Il2CppXorCodeHasher {
    /// Indicates whether SSE2 is available on the current CPU (detected at runtime).
    sse2_available: bool,

    /// A 16–byte accumulator that holds the XOR–result of processed blocks.
    accum: [u8; 16],

    /// Buffer to store bytes that do not fill a complete 16–byte block.
    leftover: [u8; 16],
    leftover_len: usize,
}

impl Il2CppXorCodeHasher {
    /// Constructs a new XorCodeHasher.
    ///
    /// On x86_64 systems, SSE2 is almost always present. However, this check makes the code
    /// portable to other architectures that may not support SSE2.
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

    /// Convenience function to compute a hash from the provided data in one call.
    pub fn hash(data: &[u8]) -> u64 {
        let mut hasher = Self::new();
        hasher.write(data);
        hasher.finish()
    }

    /// XORs a 16–byte block into the accumulator.
    ///
    /// If SSE2 is available, this operation is accelerated using SIMD instructions.
    /// Otherwise, it falls back to a simple scalar byte–by–byte XOR.
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

    /// Performs a SIMD–accelerated XOR of a 16–byte block into the accumulator using SSE2.
    ///
    /// # Safety
    /// This function uses raw pointer operations and SSE2 intrinsics, so it is marked unsafe.
    #[target_feature(enable = "sse2")]
    unsafe fn xor_16_bytes_sse2(&mut self, block: &[u8; 16]) {
        use core::arch::x86_64::*;

        // Load the current accumulator and the new block into 128–bit SIMD registers.
        let reg_accum = unsafe { _mm_loadu_si128(self.accum.as_ptr() as *const __m128i) };
        let reg_block = unsafe { _mm_loadu_si128(block.as_ptr() as *const __m128i) };

        // Compute the XOR of both registers.
        let result = unsafe { _mm_xor_si128(reg_accum, reg_block) };
        // Store the result back into the accumulator.
        unsafe { _mm_storeu_si128(self.accum.as_mut_ptr() as *mut __m128i, result) };
    }
}

impl Default for Il2CppXorCodeHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Il2CppXorCodeHasher {
    /// Finalizes the hasher and returns the checksum.
    ///
    /// The method first merges any remaining (leftover) bytes into the accumulator.
    /// It then reduces the 16–byte accumulator to a single byte by XORing all bytes,
    /// and finally zero–extends the result to a 64–bit value.
    #[inline]
    fn finish(&self) -> u64 {
        // Copy the accumulator so that self is not mutated during finalization.
        let mut accum = self.accum;

        // Merge leftover bytes into the accumulator.
        for (i, leftover) in self.leftover.iter().enumerate() {
            accum[i] ^= leftover;
        }

        // Reduce the accumulator to a single byte by XORing all its bytes.
        let mut x = 0u8;
        for &b in &accum {
            x ^= b;
        }

        // Return the final 8–bit checksum as a u64.
        x as u64
    }

    /// Streams bytes into the hasher.
    ///
    /// The process involves three steps:
    /// 1. If there is an incomplete 16–byte block (leftover from previous writes), fill it.
    /// 2. Process as many full 16–byte blocks as possible from the input directly.
    /// 3. Buffer any remaining bytes (less than 16) in the leftover array for future writes.
    fn write(&mut self, mut bytes: &[u8]) {
        // First, complete any partial block from previous writes.
        if self.leftover_len > 0 {
            let needed = 16 - self.leftover_len;
            if bytes.len() < needed {
                // Not enough bytes to complete the block; just buffer them.
                self.leftover[self.leftover_len..(self.leftover_len + bytes.len())]
                    .copy_from_slice(bytes);
                self.leftover_len += bytes.len();
                return;
            } else {
                // Fill the leftover buffer to form a complete 16–byte block.
                self.leftover[self.leftover_len..16].copy_from_slice(&bytes[..needed]);
                bytes = &bytes[needed..];

                // Process the now–complete 16–byte block.
                let block = unsafe { &*(self.leftover.as_ptr() as *const [u8; 16]) };
                self.xor_16_bytes(block);

                // Reset the leftover buffer.
                self.leftover_len = 0;
            }
        }

        // Process any full 16–byte blocks directly from the input.
        while bytes.len() >= 16 {
            let block = unsafe { &*(bytes.as_ptr() as *const [u8; 16]) };
            self.xor_16_bytes(block);
            bytes = &bytes[16..];
        }

        // Buffer any remaining bytes (less than 16) for future writes.
        self.leftover[..bytes.len()].copy_from_slice(bytes);
        self.leftover_len = bytes.len();
    }
}

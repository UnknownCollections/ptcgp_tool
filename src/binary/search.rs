use memchr::memchr_iter;

/// Searches for all occurrences of the byte pattern (`needle`) in the data slice (`data`).
///
/// This function uses an optimized approach:
/// - **Single-byte pattern:** When `needle` is one byte long, it uses `memchr_iter` to directly
///   locate all occurrences of that byte.
/// - **Multi-byte pattern:** For longer patterns, it first finds candidate starting positions
///   using the first byte, then verifies if the subsequent bytes match the entire pattern.
///
/// # Arguments
///
/// * `data` - The slice of bytes to be searched.
/// * `needle` - The byte pattern to look for.
///
/// # Returns
///
/// A vector of starting indices in `data` where the `needle` is found.
pub fn find_pattern(data: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut results = Vec::new();

    // For a single-byte needle, directly use memchr_iter for an efficient search.
    if needle.len() == 1 {
        for candidate in memchr_iter(needle[0], data) {
            results.push(candidate);
        }
    } else {
        // For multi-byte needles:
        // 1. Use memchr_iter to identify candidate positions where the first byte of the needle appears.
        // 2. Check if the slice from the candidate position is long enough and matches the full needle.
        for candidate in memchr_iter(needle[0], data) {
            if candidate + needle.len() <= data.len()
                && &data[candidate..candidate + needle.len()] == needle
            {
                results.push(candidate);
            }
        }
    }
    results
}

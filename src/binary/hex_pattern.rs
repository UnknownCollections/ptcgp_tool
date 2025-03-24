/// Converts a single hex digit (as a byte) to its value.
const fn parse_hex_digit(b: u8) -> u8 {
    match b {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'a' | b'A' => 10,
        b'b' | b'B' => 11,
        b'c' | b'C' => 12,
        b'd' | b'D' => 13,
        b'e' | b'E' => 14,
        b'f' | b'F' => 15,
        _ => 0,
    }
}

/// Parses a pattern string (tokens separated by spaces) into three values:
/// - a fixed array of bytes,
/// - a fixed array of booleans (mask), and
/// - the count of tokens found.
///
/// Each token is expected to be at least two characters long. If the two
/// characters are "??", then the byte value is set to 0 and the mask is false.
/// Otherwise, the first two characters are interpreted as hex digits.
const fn parse_hex_pattern(s: &str) -> ([u8; 256], [bool; 256], usize) {
    let bytes = s.as_bytes();
    let mut pattern = [0u8; 256];
    let mut mask = [false; 256];
    let mut count = 0;
    let mut i = 0;

    while i < bytes.len() {
        // Skip leading spaces.
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let token_start = i;
        // Advance until we hit a space (or end of string)
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }
        let token_end = i;
        let token_len = token_end - token_start;

        // If we have at least two characters, use the first two.
        if token_len >= 2 {
            let b0 = bytes[token_start];
            let b1 = bytes[token_start + 1];
            if b0 == b'?' && b1 == b'?' {
                pattern[count] = 0;
                mask[count] = false;
            } else {
                pattern[count] = (parse_hex_digit(b0) << 4) | parse_hex_digit(b1);
                mask[count] = true;
            }
        } else {
            // For tokens that are too short, return a dummy value.
            pattern[count] = 0;
            mask[count] = false;
        }
        count += 1;
    }

    (pattern, mask, count)
}

/// A type that holds a parsed pattern.
///
/// The pattern is stored in fixed-size arrays (with a max capacity of 256) computed at compile time,
/// and you can get slices corresponding to the parsed data.
pub struct HexPattern {
    pattern: [u8; 256],
    mask: [bool; 256],
    count: usize,
}

impl HexPattern {
    /// Parse the given pattern string at compile time.
    ///
    /// Example:
    ///
    /// ```rust
    /// const MY_PATTERN: HexPattern = HexPattern::new("ff ?? 00 01");
    /// ```
    pub const fn new(s: &str) -> HexPattern {
        let (pattern, mask, count) = parse_hex_pattern(s);
        HexPattern {
            pattern,
            mask,
            count,
        }
    }

    /// Returns the slice of bytes in the parsed pattern.
    pub fn pattern(&self) -> &[u8] {
        &self.pattern[..self.count]
    }

    /// Returns the slice of booleans (mask) in the parsed pattern.
    pub fn mask(&self) -> &[bool] {
        &self.mask[..self.count]
    }
}

/// Searches `data` for the first occurrence of the pattern (using `mask` to ignore wildcards)
pub fn find_hex_pattern(data: &[u8], pattern: &[u8], mask: &[bool]) -> Option<usize> {
    assert_eq!(pattern.len(), mask.len());
    if pattern.is_empty() {
        return Some(0);
    }
    for i in 0..=data.len().saturating_sub(pattern.len()) {
        // For each possible starting offset, check every byte in the pattern.
        let mut found = true;
        for j in 0..pattern.len() {
            // If mask[j] is true we require an exact match.
            if mask[j] && data[i + j] != pattern[j] {
                found = false;
                break;
            }
        }
        if found {
            return Some(i);
        }
    }
    None
}

use crate::constants;

pub mod fs;
pub mod progress;

/// Gets the default amount of parallelism to use.
///
/// If the value could not be obtained, defaults to ``crate::constants::DEFAULT_PARALLELISM``
pub fn available_parallelism() -> usize {
    if let Ok(available) = std::thread::available_parallelism() {
        available.get()
    } else {
        constants::DEFAULT_PARALLELISM
    }
}

/// Parses a hex string into a Vec of bytes.
/// 
/// Implementation credit: https://stackoverflow.com/a/52992629
pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex_str.len())
        .step_by(2)
        .map(|num| u8::from_str_radix(&hex_str[num..num + 2], 16))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utils_decode_hex() {
        let decoded = decode_hex("6732666343305a637a4e394d544a3631").unwrap();
        assert_eq!(decoded, vec![103, 50, 102, 99, 67, 48, 90, 99, 122, 78, 57, 77, 84, 74, 54, 49])
    }
}

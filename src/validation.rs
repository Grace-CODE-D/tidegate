pub const MAX_CACHE_KEY_LENGTH: usize = 128;

#[derive(Debug, PartialEq, Eq)]
pub enum CacheKeyError {
    Empty,
    TooLong,
    ContainsPathSeparator,
    ContainsControlCharacter,
    ContainsWhitespace,
}

pub fn validate_cache_key(key: &str) -> Result<(), CacheKeyError> {
    if key.is_empty() {
        return Err(CacheKeyError::Empty);
    }
    if key.chars().count() > MAX_CACHE_KEY_LENGTH {
        return Err(CacheKeyError::TooLong);
    }
    if key.contains('/') || key.contains('\\') {
        return Err(CacheKeyError::ContainsPathSeparator);
    }
    if key.chars().any(char::is_control) {
        return Err(CacheKeyError::ContainsControlCharacter);
    }
    if key.chars().any(char::is_whitespace) {
        return Err(CacheKeyError::ContainsWhitespace);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_keys() {
        assert_eq!(validate_cache_key("profile:demo_01"), Ok(()));
    }

    #[test]
    fn rejects_empty_and_whitespace_keys() {
        assert_eq!(validate_cache_key(""), Err(CacheKeyError::Empty));
        assert_eq!(
            validate_cache_key("has space"),
            Err(CacheKeyError::ContainsWhitespace)
        );
    }

    #[test]
    fn rejects_path_like_and_control_keys() {
        assert_eq!(
            validate_cache_key("../secret"),
            Err(CacheKeyError::ContainsPathSeparator)
        );
        assert_eq!(
            validate_cache_key("line\nfeed"),
            Err(CacheKeyError::ContainsControlCharacter)
        );
    }

    #[test]
    fn rejects_keys_longer_than_the_limit() {
        let key = "x".repeat(MAX_CACHE_KEY_LENGTH + 1);
        assert_eq!(validate_cache_key(&key), Err(CacheKeyError::TooLong));
    }
}

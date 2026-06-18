use std::sync::LazyLock;

use regex::Regex;
use validator::ValidationError;

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

pub fn validate_username(value: &str) -> Result<(), ValidationError> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        let e = ValidationError::new("field_is_required").with_message("Field is required.".into());
        return Err(e);
    }

    let chars_count = trimmed.chars().count();
    if !(5..=32).contains(&chars_count) {
        let e = ValidationError::new("invalid_length")
            .with_message("Username must be between 5 and 32 characters.".into());
        return Err(e);
    }

    if !USERNAME_REGEX.is_match(trimmed) {
        const USERNAME_FORMAT: &str =
            "Username can contain only Latin letters, digits, and underscores.";

        let e = ValidationError::new("invalid_format").with_message(USERNAME_FORMAT.into());
        return Err(e);
    }

    if trimmed.starts_with('_') {
        let e = ValidationError::new("invalid_format")
            .with_message("Username cannot start with an underscore.".into());
        return Err(e);
    }

    if trimmed.ends_with('_') {
        let e = ValidationError::new("invalid_format")
            .with_message("Username cannot end with an underscore.".into());
        return Err(e);
    }

    if trimmed.contains("__") {
        let e = ValidationError::new("invalid_format")
            .with_message("Username cannot contain consecutive underscores.".into());
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_username;

    #[test]
    fn valid_username_with_letters() {
        assert!(validate_username("johndoe").is_ok());
    }

    #[test]
    fn valid_username_with_letters_and_numbers() {
        assert!(validate_username("john123").is_ok());
    }

    #[test]
    fn valid_username_with_letters_numbers_and_underscores() {
        assert!(validate_username("john_doe_123").is_ok());
    }

    #[test]
    fn valid_username_min_length() {
        assert!(validate_username("abcde").is_ok());
    }

    #[test]
    fn valid_username_max_length() {
        let max_username = "a".repeat(32);
        assert!(validate_username(&max_username).is_ok());
    }

    #[test]
    fn username_too_short() {
        let err = validate_username("ab").unwrap_err();
        assert_eq!(err.code, "invalid_length");
    }

    #[test]
    fn username_too_long() {
        let long_username = "a".repeat(33);
        let err = validate_username(&long_username).unwrap_err();
        assert_eq!(err.code, "invalid_length");
    }

    #[test]
    fn username_with_hyphen() {
        let err = validate_username("john-doe").unwrap_err();
        assert_eq!(err.code, "invalid_format");
    }

    #[test]
    fn username_with_dot() {
        let err = validate_username("john.doe").unwrap_err();
        assert_eq!(err.code, "invalid_format");
    }

    #[test]
    fn username_with_numbers_only() {
        assert!(validate_username("12345").is_ok());
    }

    #[test]
    fn username_starts_with_underscore() {
        let err = validate_username("_johndoe").unwrap_err();
        assert_eq!(err.code, "invalid_format");
    }

    #[test]
    fn username_ends_with_underscore() {
        let err = validate_username("johndoe_").unwrap_err();
        assert_eq!(err.code, "invalid_format");
    }

    #[test]
    fn username_with_consecutive_underscores() {
        let err = validate_username("john__doe").unwrap_err();
        assert_eq!(err.code, "invalid_format");
    }

    #[test]
    fn username_empty() {
        let err = validate_username("").unwrap_err();
        assert_eq!(err.code, "field_is_required");
    }

    #[test]
    fn username_whitespace_only() {
        let err = validate_username("   ").unwrap_err();
        assert_eq!(err.code, "field_is_required");
    }

    #[test]
    fn username_with_leading_trailing_spaces() {
        assert!(validate_username("  johndoe  ").is_ok());
    }

    #[test]
    fn username_with_special_characters() {
        let err = validate_username("john@doe").unwrap_err();
        assert_eq!(err.code, "invalid_format");
    }
}

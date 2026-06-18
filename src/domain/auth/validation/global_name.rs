use std::sync::LazyLock;

use regex::Regex;
use validator::ValidationError;

static GLOBAL_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}\p{M}\p{Emoji}\p{So}\s\-\'\.\,!?]+$").unwrap());

pub fn validate_global_name(value: &str) -> Result<(), ValidationError> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        let e = ValidationError::new("field_is_required").with_message("Field is required.".into());
        return Err(e);
    }

    let chars_count = trimmed.chars().count();
    if !(1..=64).contains(&chars_count) {
        let e = ValidationError::new("invalid_length")
            .with_message("Global name must be between 1 and 64 characters.".into());
        return Err(e);
    }

    if !GLOBAL_NAME_REGEX.is_match(trimmed) {
        let e = ValidationError::new("invalid_format").with_message(
            "Global name can contain letters, numbers, spaces, and basic punctuation.".into(),
        );
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_global_name_with_english() {
        assert!(validate_global_name("John Doe").is_ok());
    }

    #[test]
    fn valid_global_name_with_cyrillic() {
        assert!(validate_global_name("Иван Петров").is_ok());
    }

    #[test]
    fn valid_global_name_with_emoji() {
        assert!(validate_global_name("🌟 Star User").is_ok());
    }

    #[test]
    fn valid_global_name_with_punctuation() {
        assert!(validate_global_name("John-Doe").is_ok());
        assert!(validate_global_name("O'Connor").is_ok());
        assert!(validate_global_name("Dr. John Smith, Jr.").is_ok());
        assert!(validate_global_name("John!").is_ok());
        assert!(validate_global_name("John?").is_ok());
    }

    #[test]
    fn valid_global_name_with_numbers() {
        assert!(validate_global_name("User123").is_ok());
        assert!(validate_global_name("007 James Bond").is_ok());
    }

    #[test]
    fn valid_global_name_min_length() {
        assert!(validate_global_name("a").is_ok());
    }

    #[test]
    fn valid_global_name_max_length() {
        let max_name = "a".repeat(64);
        assert!(validate_global_name(&max_name).is_ok());
    }

    #[test]
    fn global_name_empty() {
        let err = validate_global_name("").unwrap_err();
        assert_eq!(err.code, "field_is_required");
    }

    #[test]
    fn global_name_whitespace_only() {
        let err = validate_global_name("   ").unwrap_err();
        assert_eq!(err.code, "field_is_required");
    }

    #[test]
    fn global_name_too_long() {
        let long_name = "в".repeat(65);
        let err = validate_global_name(&long_name).unwrap_err();
        assert_eq!(err.code, "invalid_length");
    }

    #[test]
    fn global_name_with_leading_trailing_spaces() {
        assert!(validate_global_name("  John Doe  ").is_ok());
    }

    #[test]
    fn global_name_international() {
        assert!(validate_global_name("陈明").is_ok());
        assert!(validate_global_name("ありがとう").is_ok());
        assert!(validate_global_name("감사합니다").is_ok());
        assert!(validate_global_name("Ευχαριστώ").is_ok());
        assert!(validate_global_name("شكرًا").is_ok());
        assert!(validate_global_name("תודה").is_ok());
        assert!(validate_global_name("धन्यवाद").is_ok());
    }

    #[test]
    fn global_name_with_accents() {
        assert!(validate_global_name("José Rodríguez").is_ok());
        assert!(validate_global_name("François Chrétien").is_ok());
        assert!(validate_global_name("Mikaël Åkerlund").is_ok());
        assert!(validate_global_name("Siân O'Connor").is_ok());
    }

    #[test]
    fn global_name_with_combining_chars() {
        let name_with_combining = "c\u{0327}a va?";
        assert!(validate_global_name(name_with_combining).is_ok());
    }

    #[test]
    fn global_name_with_tabs_and_newlines() {
        assert!(validate_global_name("John\tDoe").is_ok());
        assert!(validate_global_name("John\nDoe").is_ok());
        assert!(validate_global_name("John\r\nDoe").is_ok());
    }

    #[test]
    fn global_name_with_apostrophe_edge_cases() {
        assert!(validate_global_name("O'Connor").is_ok());
        assert!(validate_global_name("D'Artagnan").is_ok());
        assert!(validate_global_name("Rock 'n' Roll").is_ok());
        assert!(validate_global_name("'Single quoted'").is_ok());
        assert!(validate_global_name("Ends with'").is_ok());
    }

    #[test]
    fn global_name_with_hyphen_edge_cases() {
        assert!(validate_global_name("-Starts with hyphen").is_ok());
        assert!(validate_global_name("Ends with hyphen-").is_ok());
        assert!(validate_global_name("Double--hyphen").is_ok());
        assert!(validate_global_name("Jean-Claude Van Damme").is_ok());
    }
}

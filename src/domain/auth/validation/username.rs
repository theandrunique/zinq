use std::sync::LazyLock;

use regex::Regex;
use validator::ValidationError;

static USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9]+$").unwrap());

pub fn validate_username(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let e = ValidationError::new("field_is_required").with_message("Field is required.".into());
        return Err(e);
    }

    if value.len() < 3 || value.len() > 30 {
        let e = ValidationError::new("invalid_length")
            .with_message("Username must be between 3 and 30 characters.".into());
        return Err(e);
    }

    if !USERNAME_REGEX.is_match(value) {
        let e = ValidationError::new("invalid_format")
            .with_message("Username must contain only letters and numbers.".into());
        return Err(e);
    }

    Ok(())
}

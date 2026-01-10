use std::sync::LazyLock;

use regex::Regex;
use validator::ValidationError;

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$").unwrap()
});

pub fn validate_email(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let e = ValidationError::new("field_is_required").with_message("Field is required.".into());
        return Err(e);
    }

    if !EMAIL_REGEX.is_match(value) {
        let e = ValidationError::new("invalid_format").with_message("Invalid email.".into());
        return Err(e);
    }

    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_email_success() {
        let valid_emails = [
            "test@example.com",
            "user.name+tag@gmail.com",
            "user_name@test-domain.org",
            "a@b.co",
        ];

        for email in valid_emails {
            assert!(validate_email(email).is_ok(), "email failed: {}", email);
        }
    }

    #[test]
    fn validate_email_empty() {
        let err = validate_email("").unwrap_err();
        assert_eq!(err.code, "field_is_required");

        let err = validate_email("   ").unwrap_err();
        assert_eq!(err.code, "field_is_required");
    }

    #[test]
    fn validate_email_invalid_format() {
        let invalid_emails = [
            "plainaddress",
            "@no-local-part.com",
            "user@",
            "user@.com",
            "user@com",
            "user@@example.com",
        ];

        for email in invalid_emails {
            let err = validate_email(email).unwrap_err();
            assert_eq!(err.code, "invalid_format", "email: {}", email);
        }
    }
}

use validator::ValidationError;

pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let e = ValidationError::new("field_is_required").with_message("Field is required.".into());
        return Err(e);
    }

    let chars_count = value.chars().count();
    if !(8..=50).contains(&chars_count) {
        let e = ValidationError::new("invalid_length")
            .with_message("Password must be between 8 and 50 characters.".into());
        return Err(e);
    }
    validate_password_complexity(value)?;

    Ok(())
}

fn validate_password_complexity(password: &str) -> Result<(), ValidationError> {
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_digit = false;
    let mut has_special = false;

    static SPECIAL_CHARS: &str = "!@#$%^&*()_+-=[]{}|;:'\",.<>?/`~";

    for c in password.chars() {
        if c.is_ascii_uppercase() {
            has_uppercase = true;
        } else if c.is_ascii_lowercase() {
            has_lowercase = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if SPECIAL_CHARS.contains(c) {
            has_special = true;
        }
    }

    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        let e = ValidationError::new("password_complexity")
            .with_message("Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character.".into());
        Err(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_password_min_length() {
        assert!(validate_password("Aa1!5678").is_ok());
    }

    #[test]
    fn password_empty() {
        let err = validate_password("").unwrap_err();
        assert_eq!(err.code, "field_is_required");
    }

    #[test]
    fn password_with_internal_spaces() {
        let result = validate_password("abc! 5A78");
        assert!(result.is_ok());
    }

    #[test]
    fn password_too_short() {
        let err = validate_password("Aa1!567").unwrap_err();
        assert_eq!(err.code, "invalid_length");
    }

    #[test]
    fn password_no_uppercase() {
        let err = validate_password("password123!").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_no_lowercase() {
        let err = validate_password("PASSWORD123!").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_no_digit() {
        let err = validate_password("Password!!").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_no_special_char() {
        let err = validate_password("Password123").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_only_uppercase() {
        let err = validate_password("ABCDEFGH").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_only_lowercase() {
        let err = validate_password("abcdefgh").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_only_digits() {
        let err = validate_password("12345678").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_only_special_chars() {
        let err = validate_password("!@#$%^&*").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }

    #[test]
    fn password_with_unicode() {
        let err = validate_password("Пароль123!").unwrap_err();
        assert_eq!(err.code, "password_complexity");
    }
}

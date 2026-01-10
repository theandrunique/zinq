use validator::ValidationError;

pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let e = ValidationError::new("field_is_required").with_message("Field is required.".into());
        return Err(e);
    }

    if value.len() < 8 || value.len() > 50 {
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

    for c in password.chars() {
        if c.is_ascii_uppercase() {
            has_uppercase = true;
        } else if c.is_ascii_lowercase() {
            has_lowercase = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if "#?!@$%^&*-".contains(c) {
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

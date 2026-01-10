use std::collections::HashMap;

use validator::Validate;

use crate::error::Error;

pub trait ValidateExt: Validate {
    fn validate(&self) -> Result<(), Error> {
        match Validate::validate(self) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let mut error_map = HashMap::new();

                for (field, field_errors) in errors.field_errors() {
                    let messages: Vec<String> = field_errors
                        .iter()
                        .map(|error| {
                            error
                                .message
                                .as_ref()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "Invalid value".to_string())
                        })
                        .collect();

                    error_map.insert(field.to_string(), messages);
                }

                Err(Error::InvalidRequestBody(error_map))
            }
        }
    }
}

impl<T: Validate> ValidateExt for T {}

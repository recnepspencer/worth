//! Flattened record paths are still single native field names.

use super::{validate_simple_identifier, ApplicationSchemaDeclarationDenial as Denial};

pub(in crate::application_schema) fn validate_field_identifier(path: &str) -> Result<(), Denial> {
    for segment in path.split('.') {
        let mut pieces = segment.split('[');
        let name = pieces.next().ok_or(Denial::InvalidIdentifier)?;
        validate_simple_identifier(name)?;
        if name.contains(']') || name.chars().any(char::is_control) {
            return Err(Denial::InvalidIdentifier);
        }
        for index in pieces {
            let digits = index.strip_suffix(']').ok_or(Denial::InvalidIdentifier)?;
            if digits.is_empty()
                || !digits.bytes().all(|digit| digit.is_ascii_digit())
                || (digits.len() > 1 && digits.starts_with('0'))
            {
                return Err(Denial::InvalidIdentifier);
            }
        }
    }
    Ok(())
}

pub(in crate::application_schema) fn validate_field_locus(
    entity: &str,
    aspect: &str,
    field: &str,
) -> Result<(), Denial> {
    validate_simple_identifier(entity)?;
    validate_simple_identifier(aspect)?;
    validate_field_identifier(field)
}

pub(super) fn portable_family(value: &str) -> bool {
    portable_identity(value) && value.contains('.')
}

pub(super) fn portable_identity(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value.chars().any(char::is_whitespace)
        && !value.chars().any(char::is_control)
}

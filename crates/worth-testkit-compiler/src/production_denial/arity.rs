//! E0061's required and supplied argument counts are semantic call shape.
pub(super) fn matches(message: &str, required: usize, supplied: usize) -> bool {
    let rest = message
        .strip_prefix("this method takes ")
        .or_else(|| message.strip_prefix("this function takes "));
    let Some(rest) = rest else { return false };
    let Some((expected, rest)) = rest.split_once(" argument") else {
        return false;
    };
    let rest = rest.strip_prefix('s').unwrap_or(rest);
    let Some(rest) = rest.strip_prefix(" but ") else {
        return false;
    };
    let Some((provided, tail)) = rest.split_once(" argument") else {
        return false;
    };
    matches!(tail, " was supplied" | "s were supplied")
        && expected.parse::<usize>().ok() == Some(required)
        && provided.parse::<usize>().ok() == Some(supplied)
}

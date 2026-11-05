//! Current expression-only persisted condition meaning. Older shapes refuse.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::condition::OPERAND_FIELDS;

/// Frames each operand's fields as `length:field`, operands in name order.
pub(in crate::domain_computation::primary_graph) fn encode_operands<'a>(
    operands: impl IntoIterator<Item = [&'a str; OPERAND_FIELDS]>,
) -> String {
    use std::fmt::Write;

    let mut text = String::new();
    for field in operands.into_iter().flatten() {
        write!(&mut text, "{}:{field}", field.len()).expect("writing to a String cannot fail");
    }
    text
}

#[cfg(test)]
#[path = "condition/tests.rs"]
mod tests;

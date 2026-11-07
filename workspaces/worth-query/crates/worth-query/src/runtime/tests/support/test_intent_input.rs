use super::*;

pub(in crate::runtime::tests) fn test_intent_input(
    fields: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> WorthQueryIntentInput {
    WorthQueryIntentInput::object(
        fields
            .into_iter()
            .map(|(field, value)| (field, WorthQueryIntentInput::string(value))),
    )
}

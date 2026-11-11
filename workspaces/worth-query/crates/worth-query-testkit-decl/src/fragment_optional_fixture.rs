//! A case whose complete row may contain no user scalar value.
use super::fragment_fixture::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionalText {
    pub text: Option<String>,
}
pub struct Noted;
impl Aspect for Noted {
    const NAME: &'static str = "noted";
    type Row = OptionalText;
}
impl Record<OptionalText> for Bindings {
    fn record() -> impl Shape<OptionalText, Self> {
        field("text", |value: &OptionalText| value.text.clone()).build(
            |text, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(OptionalText { text })
            },
        )
    }
}
impl From<OptionalText> for FormValue {
    fn from(value: OptionalText) -> Self {
        Self::Noted(value)
    }
}

pub struct Note;
impl Aspect for Note {
    const NAME: &'static str = "note";
    type Row = OptionalText;
}

pub struct RequiredNote;
impl Aspect for RequiredNote {
    const NAME: &'static str = "required_note";
    type Row = OptionalText;
}

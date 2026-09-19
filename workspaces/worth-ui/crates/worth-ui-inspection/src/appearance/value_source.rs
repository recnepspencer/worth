/// Source of a resolved appearance value; inspection data grants no authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionValueSource {
    ThemeSlot {
        selected: Box<str>,
        terminal: Box<str>,
    },
    Literal,
    Unavailable,
}

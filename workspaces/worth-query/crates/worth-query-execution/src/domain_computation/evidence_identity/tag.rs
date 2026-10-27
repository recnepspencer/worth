#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryEvidenceTag(&'static str);

impl WorthQueryEvidenceTag {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }
}

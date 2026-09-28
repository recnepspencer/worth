use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

/// The keyword that introduces an expression body: `when` for a condition,
/// `value` for a derived declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthUiExpressionIntroducer {
    When,
    Value,
}

impl WorthUiExpressionIntroducer {
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::When => "when",
            Self::Value => "value",
        }
    }
}

/// The raw kernel expression text of one declaration, with the host-file
/// byte offset of its first character.
///
/// `body_start` is provenance for mapping kernel spans back to the host
/// file. It is never meaning: equality, ordering, and hashing use only the
/// introducer and the source text.
#[derive(Clone, Debug)]
pub struct WorthUiExpressionBody {
    introducer: WorthUiExpressionIntroducer,
    source: String,
    body_start: usize,
}

impl WorthUiExpressionBody {
    pub fn new(
        introducer: WorthUiExpressionIntroducer,
        source: impl Into<String>,
        body_start: usize,
    ) -> Self {
        Self {
            introducer,
            source: source.into(),
            body_start,
        }
    }

    pub fn introducer(&self) -> WorthUiExpressionIntroducer {
        self.introducer
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn body_start(&self) -> usize {
        self.body_start
    }
}

impl PartialEq for WorthUiExpressionBody {
    fn eq(&self, other: &Self) -> bool {
        self.introducer == other.introducer && self.source == other.source
    }
}

impl Eq for WorthUiExpressionBody {}

impl PartialOrd for WorthUiExpressionBody {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorthUiExpressionBody {
    fn cmp(&self, other: &Self) -> Ordering {
        self.introducer
            .cmp(&other.introducer)
            .then_with(|| self.source.cmp(&other.source))
    }
}

impl Hash for WorthUiExpressionBody {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.introducer.hash(state);
        self.source.hash(state);
    }
}

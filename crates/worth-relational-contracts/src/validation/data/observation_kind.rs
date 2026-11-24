use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantObservationKind {
    Committed,
    Speculative,
}

impl InvariantObservationKind {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Speculative => "speculative",
        }
    }
}

use super::super::WorthQueryInvariantEntityIdentity;

mod resolution;

/// Which current entity a producer's output family holds for a producer.
pub enum WorthQueryCurrentOutputSelection<Schema, Entity> {
    /// Exactly one current entity is the output.
    Unique(WorthQueryInvariantEntityIdentity<Schema, Entity>),
    /// No current entity is the output.
    Missing,
    /// More than one current entity is the output.
    Ambiguous(Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>),
    /// The producer is no longer live, so none of its outputs are current.
    ObsoleteSource,
}

/// Why the current output for a producer could not be selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCurrentOutputDenialKind {
    /// The projection has no selected product, or the output family is not
    /// installed for this operation's binding.
    FamilyUnavailable,
    /// No recorded output is current: each was produced from source state that
    /// has since changed. A change to the producer itself reports
    /// `OutputUnavailable` instead.
    StaleSource,
    /// A recorded output entity is not of the family's entity kind.
    EntityMismatch,
    /// A recorded output entity is no longer live, the producer changed after
    /// its outputs were recorded, or a recorded source fact could not be read.
    OutputUnavailable,
    /// The operation does not declare the family's entity or the producer as
    /// a decision read.
    UndeclaredDecisionTarget,
    /// The producer identity came from a different projection authority.
    ForeignIdentity,
    /// The selection exceeded the projection's work budget.
    WorkBudgetExceeded,
}

/// Refusal to select a producer's current output.
///
/// [`Self::kind`] says why and [`Self::subject`] names the family, the
/// producer entity, or the output role read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCurrentOutputDenial {
    kind: WorthQueryCurrentOutputDenialKind,
    subject: String,
}

impl WorthQueryCurrentOutputDenial {
    pub const fn kind(&self) -> WorthQueryCurrentOutputDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    fn new(kind: WorthQueryCurrentOutputDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryCurrentOutputDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "current output denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryCurrentOutputDenial {}

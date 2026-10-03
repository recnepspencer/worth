use std::marker::PhantomData;

use super::super::WorthQueryInvariantEntityIdentity;

mod resolution;

/// A named role in a producer's output family, typed by the entity kind that
/// fills it.
///
/// Pass it to `current_output` to find the entity currently filling the role.
pub struct WorthQueryCurrentOutputRole<Family, Entity> {
    name: &'static str,
    _marker: PhantomData<fn() -> (Family, Entity)>,
}

impl<Family, Entity> Copy for WorthQueryCurrentOutputRole<Family, Entity> {}

impl<Family, Entity> Clone for WorthQueryCurrentOutputRole<Family, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Family, Entity> WorthQueryCurrentOutputRole<Family, Entity> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }
}

/// Which current entity fills an output role for a producer.
pub enum WorthQueryCurrentOutputSelection<Schema, Entity> {
    /// Exactly one current entity fills the role.
    Unique(WorthQueryInvariantEntityIdentity<Schema, Entity>),
    /// No current entity fills the role.
    Missing,
    /// More than one current entity fills the role.
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
    /// A recorded output entity is not of the role's entity kind.
    EntityMismatch,
    /// A recorded output entity is no longer live, the producer changed after
    /// its outputs were recorded, or a recorded source fact could not be read.
    OutputUnavailable,
    /// A consumed upstream output is pending source-aligned revalidation.
    PendingUpstream,
    /// Source marking changed while a verified dirty mark was being cleared.
    /// The exact companion edit stop is retained for retry policy.
    CurrentnessRaced(worth_relational::facade::mvcc::CompanionCellEditStop),
    /// The operation does not declare the role's entity or the producer as a
    /// decision read.
    UndeclaredDecisionTarget,
    /// The producer identity came from a different projection authority.
    ForeignIdentity,
    /// The selection exceeded the projection's work budget.
    WorkBudgetExceeded,
}

/// Refusal to select a producer's current output.
///
/// [`Self::kind`] says why and [`Self::subject`] names the family or role.
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

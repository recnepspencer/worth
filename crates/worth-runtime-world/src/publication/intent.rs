use std::marker::PhantomData;

use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalTransactionIntent,
};

#[path = "intent/prepared.rs"]
mod prepared;

pub(crate) use prepared::CompositePublicationStage;
pub use prepared::{
    PreparedCompositePublicationWithSignal, PreparedCompositePublicationWithoutSignal, WithSignal,
    WithoutSignal,
};

/// Which owner components a future operation is allowed to change. Omission
/// is not interpreted as an implicit refresh or a latest-head lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeComponentIntent {
    RelationalOnly(RelationalTransactionIntent),
    SignalOnly,
    RelationalAndSignal(RelationalTransactionIntent),
}

impl CompositeComponentIntent {
    pub fn relational_only(change: RelationalTransactionIntent) -> Self {
        Self::RelationalOnly(change)
    }

    pub const fn signal_only() -> Self {
        Self::SignalOnly
    }

    pub fn relational_and_signal(change: RelationalTransactionIntent) -> Self {
        Self::RelationalAndSignal(change)
    }

    pub const fn changes_relational(&self) -> bool {
        matches!(self, Self::RelationalOnly(_) | Self::RelationalAndSignal(_))
    }

    pub const fn changes_signal(&self) -> bool {
        matches!(self, Self::SignalOnly | Self::RelationalAndSignal(_))
    }

    pub fn relational_change(&self) -> Option<&RelationalTransactionIntent> {
        match self {
            Self::RelationalOnly(change) | Self::RelationalAndSignal(change) => Some(change),
            Self::SignalOnly => None,
        }
    }
}

/// The only caller-facing publication meaning. The stage parameter is the
/// compile-visible Signal decision: `WithoutSignal` is an explicit Signal
/// `RetainExact`, never an omitted plan.
#[derive(Debug)]
#[must_use = "a publication intent is prepared or dropped"]
pub struct CompositePublicationIntent<S> {
    change: CompositeComponentIntent,
    prepared_relational_candidate: Option<PreparedRelationalCommitCandidate>,
    successor_observation_requested: bool,
    _stage: PhantomData<S>,
}

impl CompositePublicationIntent<WithoutSignal> {
    /// A Relational change with an explicit Signal `RetainExact`. Both
    /// components retained is denied pre-effect, so there is no empty
    /// constructor.
    pub fn without_signal(change: RelationalTransactionIntent) -> Self {
        Self {
            change: CompositeComponentIntent::RelationalOnly(change),
            prepared_relational_candidate: None,
            successor_observation_requested: false,
            _stage: PhantomData,
        }
    }
}

impl CompositePublicationIntent<WithSignal> {
    /// An admitted Signal `AdvanceExact`, optionally alongside a Relational
    /// change.
    pub fn with_signal(change: Option<RelationalTransactionIntent>) -> Self {
        let change = match change {
            Some(change) => CompositeComponentIntent::RelationalAndSignal(change),
            None => CompositeComponentIntent::SignalOnly,
        };
        Self {
            change,
            prepared_relational_candidate: None,
            successor_observation_requested: false,
            _stage: PhantomData,
        }
    }
}

impl<S> CompositePublicationIntent<S> {
    /// Reserve and carry the exact successor observation in the performed
    /// delivery. Callers request this only when an admitted downstream lane
    /// already owns bounded capacity for that observation.
    pub fn with_successor_observation(mut self) -> Self {
        self.successor_observation_requested = true;
        self
    }

    /// Attach the one owner-issued Relational candidate that corresponds to
    /// this intent. The candidate remains move-only and is consumed by plan
    /// lowering or dropped with the intent on a rejected route.
    pub fn with_prepared_relational_candidate(
        mut self,
        candidate: PreparedRelationalCommitCandidate,
    ) -> Self {
        self.prepared_relational_candidate = Some(candidate);
        self
    }

    pub fn component_intent(&self) -> &CompositeComponentIntent {
        &self.change
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        CompositeComponentIntent,
        Option<PreparedRelationalCommitCandidate>,
        bool,
    ) {
        (
            self.change,
            self.prepared_relational_candidate,
            self.successor_observation_requested,
        )
    }
}

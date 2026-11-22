//! Pure post-commit currentness value & denial types.

pub mod diagnostic;
pub mod failure;

pub use diagnostic::RebaseVerificationReason;
pub use failure::RebaseVerificationFailure;

use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnEffectOnReads(pub OwnEffect);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OwnEffect {
    Unmoved,
    Undecidable,
    Moved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactlessCurrentness {
    Current,
    Superseded,
    Undecidable,
}

impl OwnEffectOnReads {
    pub const NONE_ASKED: Self = Self(OwnEffect::Unmoved);

    pub const fn new(effect: OwnEffect) -> Self {
        Self(effect)
    }

    pub fn join(self, effect: OwnEffect) -> Self {
        Self(self.0.max(effect))
    }

    pub const fn at_own_publication(self, selected: bool) -> FactlessCurrentness {
        match (selected, self.0) {
            (true, OwnEffect::Unmoved) => FactlessCurrentness::Current,
            (true, OwnEffect::Undecidable) => FactlessCurrentness::Undecidable,
            (true, OwnEffect::Moved) | (false, _) => FactlessCurrentness::Superseded,
        }
    }
}

#[derive(Debug)]
pub enum PreparedRebaseDenial {
    Admission(CompanionPreflightStop),
    Retention(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial),
    Temporary(std::collections::TryReserveError),
}

impl From<crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>
    for PreparedRebaseDenial
{
    fn from(
        denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
    ) -> Self {
        Self::Retention(denial)
    }
}

impl From<worth_execution::ExecutionAllocationDenial> for PreparedRebaseDenial {
    fn from(denial: worth_execution::ExecutionAllocationDenial) -> Self {
        Self::Retention(denial.into())
    }
}

impl From<std::collections::TryReserveError> for PreparedRebaseDenial {
    fn from(denial: std::collections::TryReserveError) -> Self {
        Self::Temporary(denial)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailedRebase {
    pub reason: RebaseVerificationFailure,
    pub own_effect: OwnEffectOnReads,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebasedSourceFacts {
    Exact(crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload),
    Cached {
        posting_continuation: Option<crate::domain_computation::primary_graph::output_lineage::invalidation::PostingContinuation>,
        facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        payload_bytes: u64,
        ordinals: std::sync::Arc<worth_execution::ExecutionArray<Option<usize>>>,
        superseded_count: usize,
    },
    SupersededByOwnEffect {
        facts: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        ordinals: std::sync::Arc<[usize]>,
    },
    VerificationRequired {
        reason: RebaseVerificationFailure,
        own_effect: OwnEffectOnReads,
    },
}

impl RebasedSourceFacts {
    pub fn posting_continuation(
        &self,
    ) -> Option<
        crate::domain_computation::primary_graph::output_lineage::invalidation::PostingContinuation,
    > {
        match self {
            Self::Cached {
                posting_continuation,
                ..
            } => posting_continuation.clone(),
            _ => None,
        }
    }

    pub fn payload_bytes(&self) -> Option<u64> {
        match self {
            Self::Cached { payload_bytes, .. } => Some(*payload_bytes),
            _ => None,
        }
    }

    pub fn retained(
        &self,
    ) -> Result<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        FailedRebase,
    > {
        match self {
            Self::Exact(facts)
            | Self::Cached { facts, .. }
            | Self::SupersededByOwnEffect { facts, .. } => Ok(facts.clone()),
            Self::VerificationRequired { reason, own_effect } => Err(FailedRebase {
                reason: reason.clone(),
                own_effect: *own_effect,
            }),
        }
    }

    pub fn retain_exact(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload>
    {
        self.retained().ok()
    }

    pub fn superseded_by_own_effect(&self) -> impl Iterator<Item = usize> + '_ {
        let (full, selected): (Option<&[usize]>, Option<&[Option<usize>]>) = match self {
            Self::Cached {
                ordinals,
                superseded_count,
                ..
            } => (None, (*superseded_count > 0).then(|| ordinals.elements())),
            Self::SupersededByOwnEffect { ordinals, .. } => (Some(ordinals.as_ref()), None),
            Self::Exact(_) | Self::VerificationRequired { .. } => (None, None),
        };
        full.into_iter().flatten().copied().chain(
            selected
                .into_iter()
                .flatten()
                .filter_map(|ordinal| *ordinal),
        )
    }
}

mod court_doors_for_test;

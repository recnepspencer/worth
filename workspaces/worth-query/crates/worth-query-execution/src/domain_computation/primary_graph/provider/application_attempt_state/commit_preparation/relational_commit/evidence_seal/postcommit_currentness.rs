use crate::domain_computation::primary_graph::application_checkpoint::facts::{
    FactPayload, NativeFactPayloadBuilder,
};
use std::{collections::BTreeSet, sync::Arc};

use worth_relational::facade::{identity::EntityId, mvcc::CompanionPreflightStop};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::fact_verification_observation;

mod adjacency;
mod resolution;
mod retirement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RebasedSourceFacts {
    Exact(FactPayload),
    /// Exact selected native rebase, paired with the unchanged owner capacity rule.
    Cached {
        posting_continuation: Option<crate::domain_computation::primary_graph::output_lineage::invalidation::PostingContinuation>,
        facts: FactPayload,
        payload_bytes: u64,
        ordinals: Arc<worth_execution::ExecutionArray<Option<usize>>>,
        superseded_count: usize,
    },
    /// Every fact is exact, and the committed effect moved the facts at these
    /// ordinals past what was read: a source query's read past its revision,
    /// or a field an owner call of the producer's partitioned computation
    /// read before the effect replaced it.
    SupersededByOwnEffect {
        facts: FactPayload,
        ordinals: Arc<[usize]>,
    },
    /// The rebase failed. It fails as a whole, so the commit retains none of
    /// its facts: one left as its handler read it has no native revision,
    /// and no verifier compares it. One thing is carried out instead: what
    /// the committed effect did to the facts its source query read.
    VerificationRequired {
        reason: RebaseVerificationReason,
        own_effect: OwnEffectOnReads,
    },
}

/// What a commit's own effect did to the facts its source query read, where
/// its rebase failed. Only the rebase's walk builds one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct OwnEffectOnReads(OwnEffect);

/// The walk's answer, ordered as answers join: one moved read decides it, and
/// one the walk asked about and could not compare leaves it undecidable.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum OwnEffect {
    /// No read the walk asked about moved. It asks about a producer
    /// commit's source reads, and the facts an owner call read that the
    /// effect replaced are moved before it asks: every other fact is
    /// observed again at the committed snapshot, and a non-producer's source
    /// reads are not its question.
    Unmoved,
    /// The comparison of a read the walk asked about could not answer.
    Undecidable,
    /// A read moved, or the walk's meter stopped before it could ask.
    Moved,
}

/// What a commit that kept no fact answers at an observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum FactlessCurrentness {
    /// Its own publication is selected and its effect moved none of its reads.
    Current,
    /// A later publication is selected, its effect moved one of its reads, or
    /// its rebase's meter stopped before asking about one.
    Superseded,
    /// Its own publication is selected, and the comparison of a read its
    /// rebase asked about could not answer. No observation answers it, and a
    /// recompute's own commit would meet the same comparison, so it is not
    /// superseded.
    Undecidable,
}

/// A rebase that failed: why, and what the commit's own effect did to its
/// reads. No caller holds the one without the other.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct FailedRebase {
    pub(in crate::domain_computation::primary_graph) reason: RebaseVerificationReason,
    pub(in crate::domain_computation::primary_graph) own_effect: OwnEffectOnReads,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RebaseVerificationReason {
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable(usize),
    IndexedSelectionDenied(crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(usize, crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial),
    UnsupportedDecisionFact,
    NativePayloadStorage { class: worth_execution::ExecutionAllocationDenialClass, requested: Option<u64> },
    AdmissionDenied(CompanionPreflightStop),
}

mod full_preparation;
use full_preparation::FullSourceFactRebase;
mod preparation;
mod selected;
use preparation::Preparation;
pub(in crate::domain_computation::primary_graph) struct PreparedSourceFactRebase {
    preparation: Preparation,
}

impl OwnEffectOnReads {
    /// The walk before it has asked about any read: none has moved.
    const NONE_ASKED: Self = Self(OwnEffect::Unmoved);

    fn join(self, effect: OwnEffect) -> Self {
        Self(self.0.max(effect))
    }

    /// The answer once the walk leaves `unanswered` without an answer: each
    /// read it would have asked about joins as `effect`, and one it never
    /// asks about stays out of the answer.
    fn with_unanswered(
        self,
        unanswered: &[WorthQueryApplicationObservedFact],
        producer_output: bool,
        effect: OwnEffect,
    ) -> Self {
        if producer_output
            && unanswered
                .iter()
                .inspect(|_| fact_verification_observation::rebase_unanswered())
                .any(resolution::reads_source)
        {
            self.join(effect)
        } else {
            self
        }
    }

    /// The commit's answer where `selected` says whether its own publication
    /// is the one selected.
    pub(in crate::domain_computation::primary_graph) const fn at_own_publication(
        self,
        selected: bool,
    ) -> FactlessCurrentness {
        match (selected, self.0) {
            (true, OwnEffect::Unmoved) => FactlessCurrentness::Current,
            (true, OwnEffect::Undecidable) => FactlessCurrentness::Undecidable,
            (true, OwnEffect::Moved) | (false, _) => FactlessCurrentness::Superseded,
        }
    }
}

impl RebasedSourceFacts {
    pub(super) fn posting_continuation(
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
    pub(super) fn payload_bytes(&self) -> Option<u64> {
        match self {
            Self::Cached { payload_bytes, .. } => Some(*payload_bytes),
            _ => None,
        }
    }
    /// The facts the commit retains, or the rebase that failed and left it
    /// none.
    pub(super) fn retained(&self) -> Result<FactPayload, FailedRebase> {
        match self {
            Self::Exact(facts)
            | Self::Cached { facts, .. }
            | Self::SupersededByOwnEffect { facts, .. } => Ok(facts.clone()),
            Self::VerificationRequired { reason, own_effect } => Err(FailedRebase {
                reason: *reason,
                own_effect: *own_effect,
            }),
        }
    }

    #[cfg(test)]
    pub(super) fn retain_exact(&self) -> Option<FactPayload> {
        self.retained().ok()
    }

    pub(super) fn superseded_by_own_effect(&self) -> impl Iterator<Item = usize> + '_ {
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

/// Rebase the candidate's facts onto the snapshot its own commit selected.
/// An output entity this commit retired is read as that retirement; it is not
/// a source the effect moved.
#[allow(clippy::too_many_arguments)]
pub(super) fn rebase_output(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    changed_records: &[worth_relational::facade::transactions::RecordRef],
    producer_output: bool,
    maximum_indexed_rebase_work: Option<usize>,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
) -> RebasedSourceFacts {
    let changed_entities = changed_records
        .iter()
        .filter_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let retired = correspondence
        .retired_entity_ids()
        .filter(|entity| changed_entities.contains(entity))
        .collect();
    preparation::rebase(
        execution,
        runtime,
        snapshot,
        prepared,
        &retired,
        producer_output,
        maximum_indexed_rebase_work,
        admission,
        control,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn rebase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    retired: &BTreeSet<EntityId>,
    producer: bool,
    maximum: Option<usize>,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
) -> RebasedSourceFacts {
    preparation::rebase(execution, runtime, snapshot, prepared, retired, producer, maximum, admission,
        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), None))
}
mod rebase_walk;

#[cfg(test)]
#[path = "postcommit_currentness/tests.rs"]
mod tests;
#[cfg(test)]
mod undecided_tests;

/// Exact physical source refusal stays distinct from legacy temporary Vec refusal.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum PreparedRebaseDenial {
    Admission(CompanionPreflightStop),
    Retention(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial),
    Temporary(std::collections::TryReserveError),
}
impl From<crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial> for PreparedRebaseDenial {
    fn from(denial: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial) -> Self { Self::Retention(denial) }
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

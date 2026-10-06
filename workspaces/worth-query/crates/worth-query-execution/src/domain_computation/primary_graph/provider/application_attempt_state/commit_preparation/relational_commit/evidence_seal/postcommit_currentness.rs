use std::{collections::BTreeSet, sync::Arc};

use worth_relational::facade::{identity::EntityId, mvcc::CompanionPreflightStop};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

mod adjacency;
mod resolution;
mod retirement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RebasedSourceFacts {
    Exact(Arc<[WorthQueryApplicationObservedFact]>),
    /// Every fact is exact, and the committed effect moved the source facts at
    /// these ordinals past the revision their source query read.
    SupersededByOwnEffect {
        facts: Arc<[WorthQueryApplicationObservedFact]>,
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
    /// commit's source reads only: every other fact is observed again at
    /// the committed snapshot, and a non-producer's source reads are not its
    /// question.
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
    NativeRevisionUnavailable,
    UnsupportedDecisionFact,
    AdmissionDenied(CompanionPreflightStop),
}

/// Capacity for the selected-snapshot fact transition is acquired while the
/// candidate is still pre-effect custody. The postcommit path only fills the
/// already allocated action and result buffers.
pub(in crate::domain_computation::primary_graph::provider) struct PreparedSourceFactRebase {
    facts: Vec<WorthQueryApplicationObservedFact>,
    actions: Vec<resolution::PreparedFactRebase>,
    rebased: Vec<WorthQueryApplicationObservedFact>,
    superseded: Vec<usize>,
}

impl PreparedSourceFactRebase {
    pub(in crate::domain_computation::primary_graph::provider) fn admit(
        facts: Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<Self, std::collections::TryReserveError> {
        let mut actions = Vec::new();
        actions.try_reserve_exact(facts.len())?;
        let mut rebased = Vec::new();
        rebased.try_reserve_exact(facts.len())?;
        let mut superseded = Vec::new();
        superseded.try_reserve_exact(facts.len())?;
        Ok(Self {
            facts,
            actions,
            rebased,
            superseded,
        })
    }
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
        if producer_output && unanswered.iter().any(resolution::reads_source) {
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
    /// The facts the commit retains, or the rebase that failed and left it
    /// none.
    pub(super) fn retained(
        &self,
    ) -> Result<Arc<[WorthQueryApplicationObservedFact]>, FailedRebase> {
        match self {
            Self::Exact(facts) | Self::SupersededByOwnEffect { facts, .. } => Ok(Arc::clone(facts)),
            Self::VerificationRequired { reason, own_effect } => Err(FailedRebase {
                reason: *reason,
                own_effect: *own_effect,
            }),
        }
    }

    #[cfg(test)]
    pub(super) fn retain_exact(&self) -> Option<Arc<[WorthQueryApplicationObservedFact]>> {
        self.retained().ok()
    }

    pub(super) fn superseded_by_own_effect(&self) -> &[usize] {
        match self {
            Self::SupersededByOwnEffect { ordinals, .. } => ordinals,
            Self::Exact(_) | Self::VerificationRequired { .. } => &[],
        }
    }
}

/// Rebase the candidate's facts onto the snapshot its own commit selected.
/// An output entity this commit retired is read as that retirement; it is not
/// a source the effect moved.
#[allow(clippy::too_many_arguments)]
pub(super) fn rebase_output(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    changed_records: &[worth_relational::facade::transactions::RecordRef],
    producer_output: bool,
    maximum_indexed_rebase_work: usize,
    admission: Option<&mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission>,
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
    rebase(
        runtime,
        snapshot,
        prepared,
        &retired,
        producer_output,
        maximum_indexed_rebase_work,
        admission,
    )
}

fn rebase(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    retired: &BTreeSet<EntityId>,
    producer_output: bool,
    maximum_indexed_rebase_work: usize,
    mut admission: Option<&mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission>,
) -> RebasedSourceFacts {
    let PreparedSourceFactRebase {
        facts,
        mut actions,
        mut rebased,
        mut superseded,
    } = prepared;
    let mut indexed_work = maximum_indexed_rebase_work;
    let mut failure = None;
    let mut own_effect = OwnEffectOnReads::NONE_ASKED;
    for (ordinal, fact) in facts.iter().enumerate() {
        // Past a failure the walk decides one thing: what the effect did to
        // each fact a source query read.
        if failure.is_some() && !resolution::reads_source(fact) {
            continue;
        }
        let unasked = &facts[ordinal..];
        // The fact reserves the most its resolution can spend, and settles
        // at what it spent.
        let mut spent = resolution::reserved_work(fact, producer_output);
        let mut reservation = match admission
            .as_deref_mut()
            .map(|meter| meter.reserve_external_work(spent as u64))
            .transpose()
        {
            Ok(reservation) => reservation,
            Err(stop) => return stopped(failure, stop, own_effect, unasked, producer_output),
        };
        let adjacency_work = reservation
            .as_mut()
            .map_or(0, |reserved| reserved.admission().remaining_work());
        if matches!(
            fact,
            WorthQueryApplicationObservedFact::Relation { .. }
                | WorthQueryApplicationObservedFact::Adjacency { .. }
                | WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
                | WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
        ) {
            if let Some(reserved) = reservation.as_mut() {
                // The selected native adjacency revision and the indexed
                // selection each perform one indexed probe. A probe's
                // per-call cap alone does not spend request work.
                if let Err(stop) = reserved.admission().charge_external_work(1) {
                    return stopped(failure, stop, own_effect, unasked, producer_output);
                }
            }
        }
        let prepared = resolution::PreparedFactRebase::prepare(
            runtime,
            snapshot,
            fact,
            retired,
            producer_output,
            adjacency_work,
            &mut indexed_work,
            &mut spent,
        );
        if let Some(reserved) = reservation {
            if let Err(stop) = reserved.settle(spent as u64) {
                return stopped(failure, stop, own_effect, unasked, producer_output);
            }
        }
        match prepared {
            Ok(action) => {
                if matches!(action, resolution::PreparedFactRebase::KeepSuperseded) {
                    own_effect = own_effect.join(OwnEffect::Moved);
                }
                if failure.is_none() {
                    actions.push(action);
                }
            }
            Err(reason) => {
                own_effect = own_effect.with_unanswered(
                    std::slice::from_ref(fact),
                    producer_output,
                    OwnEffect::Undecidable,
                );
                failure.get_or_insert(reason);
            }
        }
    }
    if let Some(reason) = failure {
        return RebasedSourceFacts::VerificationRequired { reason, own_effect };
    }
    for (ordinal, (fact, action)) in facts.into_iter().zip(actions).enumerate() {
        if matches!(action, resolution::PreparedFactRebase::KeepSuperseded) {
            superseded.push(ordinal);
        }
        rebased.push(action.apply(fact));
    }
    if superseded.is_empty() {
        RebasedSourceFacts::Exact(rebased.into())
    } else {
        RebasedSourceFacts::SupersededByOwnEffect {
            facts: rebased.into(),
            ordinals: superseded.into(),
        }
    }
}

/// A meter stop ends the walk and answers nothing about the reads it leaves,
/// so each one the walk would have asked about counts as moved. The commit is
/// superseded and refreshes, and its recompute pays for its own rebase. Only
/// a comparison that cannot answer leaves the effect undecidable.
fn stopped(
    failure: Option<RebaseVerificationReason>,
    stop: CompanionPreflightStop,
    own_effect: OwnEffectOnReads,
    unasked: &[WorthQueryApplicationObservedFact],
    producer_output: bool,
) -> RebasedSourceFacts {
    RebasedSourceFacts::VerificationRequired {
        reason: failure.unwrap_or(RebaseVerificationReason::AdmissionDenied(stop)),
        own_effect: own_effect.with_unanswered(unasked, producer_output, OwnEffect::Moved),
    }
}

#[cfg(test)]
#[path = "postcommit_currentness/tests.rs"]
mod tests;
#[cfg(test)]
mod undecided_tests;

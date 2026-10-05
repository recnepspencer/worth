use std::{collections::BTreeSet, sync::Arc};

use worth_relational::facade::identity::EntityId;

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

/// Whether a commit's own effect moved a fact its source query read. A read
/// the rebase could not decide counts as moved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum OwnEffectOnReads {
    Unmoved,
    Moved,
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
    AdmissionDenied(worth_relational::facade::mvcc::CompanionPreflightStop),
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
    /// The answer once the walk leaves `undecided` unread: a fact a source
    /// query read that nothing decided counts as moved.
    fn with_undecided(self, undecided: &[WorthQueryApplicationObservedFact]) -> Self {
        if undecided.iter().any(resolution::reads_source) {
            Self::Moved
        } else {
            self
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
    let mut own_effect = OwnEffectOnReads::Unmoved;
    for (ordinal, fact) in facts.iter().enumerate() {
        // Past a failure the walk decides one thing: what the effect did to
        // each fact a source query read.
        if failure.is_some() && !resolution::reads_source(fact) {
            continue;
        }
        if let Some(meter) = admission.as_mut() {
            if let Err(stop) = meter.charge_external_work(1) {
                return RebasedSourceFacts::VerificationRequired {
                    reason: failure.unwrap_or(RebaseVerificationReason::AdmissionDenied(stop)),
                    own_effect: own_effect.with_undecided(&facts[ordinal..]),
                };
            }
        }
        let adjacency_work = admission.as_ref().map_or(0, |meter| meter.remaining_work());
        if matches!(
            fact,
            WorthQueryApplicationObservedFact::Relation { .. }
                | WorthQueryApplicationObservedFact::Adjacency { .. }
                | WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
                | WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
        ) {
            if let Some(meter) = admission.as_mut() {
                // The selected native adjacency revision and the indexed
                // selection each perform one indexed probe. A probe's
                // per-call cap alone does not spend request work.
                if let Err(stop) = meter.charge_external_work(1) {
                    return RebasedSourceFacts::VerificationRequired {
                        reason: RebaseVerificationReason::AdmissionDenied(stop),
                        own_effect: own_effect.with_undecided(&facts[ordinal..]),
                    };
                }
            }
        }
        match resolution::PreparedFactRebase::prepare(
            runtime,
            snapshot,
            fact,
            retired,
            producer_output,
            adjacency_work,
            &mut indexed_work,
        ) {
            Ok(action) => {
                if matches!(action, resolution::PreparedFactRebase::KeepSuperseded) {
                    own_effect = OwnEffectOnReads::Moved;
                }
                if failure.is_none() {
                    actions.push(action);
                }
            }
            Err(reason) => {
                own_effect = own_effect.with_undecided(std::slice::from_ref(fact));
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
#[cfg(test)]
#[path = "postcommit_currentness/tests.rs"]
mod tests;

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
    VerificationRequired {
        reason: RebaseVerificationReason,
        facts: Arc<[WorthQueryApplicationObservedFact]>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RebaseVerificationReason {
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable(usize),
    IndexedSelectionDenied(crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(usize, crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial),
    UnsupportedDecisionFact,
    AdmissionDenied(worth_relational::facade::mvcc::CompanionPreflightStop),
}

/// Capacity for the selected-snapshot fact transition is acquired while the
/// candidate is still pre-effect custody. The postcommit path only fills the
/// already allocated action and result buffers.
pub(in crate::domain_computation::primary_graph) struct PreparedSourceFactRebase {
    facts: Vec<WorthQueryApplicationObservedFact>,
    endpoints: worth_execution::ExecutionArray<(
        usize,
        crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
    )>,
    actions: Vec<resolution::PreparedFactRebase>,
    rebased: Vec<WorthQueryApplicationObservedFact>,
    superseded: Vec<usize>,
}

impl PreparedSourceFactRebase {
    pub(in crate::domain_computation::primary_graph) fn admit(
        facts: Vec<WorthQueryApplicationObservedFact>,
        control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        control.check_live()?;
        let endpoint_count = facts.iter().try_fold(0usize, |count, fact| {
            control.check_live()?;
            if matches!(fact, WorthQueryApplicationObservedFact::Relation { .. } | WorthQueryApplicationObservedFact::Adjacency { .. }) {
                count.checked_add(1).ok_or(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Representability)
            } else { Ok(count) }
        })?;
        let mut endpoints =
            worth_execution::ExecutionArrayBuilder::allocate(endpoint_count, control.policy())?;
        for (ordinal, fact) in facts.iter().enumerate() {
            control.check_live()?;
            if let Some(payload) = adjacency::prepare_endpoints(fact, control)? {
                endpoints.push((ordinal, payload))?;
            }
        }
        let endpoints = endpoints.seal()?;
        let mut actions = Vec::new();
        actions.try_reserve_exact(facts.len())?;
        let mut rebased = Vec::new();
        rebased.try_reserve_exact(facts.len())?;
        let mut superseded = Vec::new();
        superseded.try_reserve_exact(facts.len())?;
        Ok(Self {
            facts,
            endpoints,
            actions,
            rebased,
            superseded,
        })
    }
}

impl RebasedSourceFacts {
    /// The same sealed facts under a requirement to verify them in full: the
    /// row a commit leaves when its facts can be compared but were not
    /// sealed as exact.
    #[cfg(feature = "test-primary-graph-faults")]
    pub(super) fn held_for_verification(self) -> Self {
        match self {
            Self::Exact(facts) | Self::SupersededByOwnEffect { facts, .. } => {
                Self::VerificationRequired {
                    reason: RebaseVerificationReason::NativeRevisionUnavailable,
                    facts,
                }
            }
            required @ Self::VerificationRequired { .. } => required,
        }
    }
    pub(super) fn retain_exact(&self) -> Option<Arc<[WorthQueryApplicationObservedFact]>> {
        match self {
            Self::Exact(facts) | Self::SupersededByOwnEffect { facts, .. } => {
                Some(Arc::clone(facts))
            }
            Self::VerificationRequired { .. } => None,
        }
    }

    pub(super) fn superseded_by_own_effect(&self) -> &[usize] {
        match self {
            Self::SupersededByOwnEffect { ordinals, .. } => ordinals,
            Self::Exact(_) | Self::VerificationRequired { .. } => &[],
        }
    }

    pub(super) fn retain_verification_facts(
        &self,
    ) -> Option<Arc<[WorthQueryApplicationObservedFact]>> {
        match self {
            Self::Exact(_) | Self::SupersededByOwnEffect { .. } => None,
            Self::VerificationRequired { facts, .. } => Some(Arc::clone(facts)),
        }
    }

    pub(super) const fn verification_requirement(&self) -> Option<RebaseVerificationReason> {
        match self {
            Self::Exact(_) | Self::SupersededByOwnEffect { .. } => None,
            Self::VerificationRequired { reason, .. } => Some(*reason),
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
        endpoints,
        mut actions,
        mut rebased,
        mut superseded,
    } = prepared;
    let mut endpoints = endpoints.into_iter().peekable();
    let mut indexed_work = maximum_indexed_rebase_work;
    for (ordinal, fact) in facts.iter().enumerate() {
        if let Some(meter) = admission.as_mut() {
            if let Err(stop) = meter.charge_external_work(1) {
                return RebasedSourceFacts::VerificationRequired {
                    reason: RebaseVerificationReason::AdmissionDenied(stop),
                    facts: facts.into(),
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
                        facts: facts.into(),
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
            endpoints
                .next_if(|(prepared_ordinal, _)| *prepared_ordinal == ordinal)
                .map(|(_, payload)| payload),
        ) {
            Ok(action) => actions.push(action),
            Err(reason) => {
                let reason = match reason {
                    RebaseVerificationReason::NativeRevisionUnavailable => {
                        RebaseVerificationReason::NativeFactRevisionUnavailable(ordinal)
                    }
                    RebaseVerificationReason::IndexedSelectionDenied(denial) => {
                        RebaseVerificationReason::IndexedSelectionFactDenied(ordinal, denial)
                    }
                    reason => reason,
                };
                return RebasedSourceFacts::VerificationRequired {
                    reason,
                    facts: facts.into(),
                };
            }
        }
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

/// Exact physical source refusal stays distinct from legacy temporary Vec refusal.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum PreparedRebaseDenial {
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

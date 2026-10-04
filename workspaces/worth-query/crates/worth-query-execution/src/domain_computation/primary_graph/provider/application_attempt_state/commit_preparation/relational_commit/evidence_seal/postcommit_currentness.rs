use std::sync::Arc;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

mod adjacency;
mod resolution;

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

impl RebasedSourceFacts {
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

pub(super) fn rebase(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    producer_output: bool,
    mut admission: Option<&mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission>,
) -> RebasedSourceFacts {
    let PreparedSourceFactRebase {
        facts,
        mut actions,
        mut rebased,
        mut superseded,
    } = prepared;
    for fact in &facts {
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
        ) {
            if let Some(meter) = admission.as_mut() {
                // The selected native adjacency revision performs one indexed
                // probe. Its per-call cap alone does not spend request work.
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
            producer_output,
            adjacency_work,
        ) {
            Ok(action) => actions.push(action),
            Err(reason) => {
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

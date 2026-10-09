use worth_relational::facade::mvcc::CompanionPreflightStop;
mod comparison;

use super::{FactMovement, WorthQueryApplicationObservedFact};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

/// Why a fact comparison gave no answer. The meter that pays for the
/// comparison stops separately, as a `CompanionPreflightStop`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQuerySourceCurrentnessFailure {
    /// The fact's own recorded bound refuses the comparison.
    WorkBudgetExceeded,
    Unavailable,
}

impl WorthQueryApplicationObservedFact {
    /// The most comparing this fact can cost: its probe, and for a native
    /// probe that found nothing the liveness check that answers it
    /// (`unanswered_probe`). A relation fact records no bound of its own, so
    /// only the meter bounds it.
    fn most_comparison_work(&self) -> usize {
        match self {
            Self::RetiredOutputEntity { .. }
            | Self::SourceEntity { .. }
            | Self::Entity { .. }
            | Self::WorkflowHistoryBasis { .. }
            | Self::WorkflowDefinitionPredecessor { lineage: None, .. } => 1,
            // A revision recorded with no comparison work cannot be compared.
            Self::SourceAdjacencyRevision {
                comparison_work_limit: 0,
                ..
            } => 0,
            Self::SourceFieldRevision { .. } | Self::SourceAdjacencyRevision { .. } => 2,
            Self::SourceAspectRevision { aspect, .. } => aspect.as_str().len().saturating_add(2),
            Self::Adjacency {
                maximum_work_units, ..
            }
            | Self::WorkflowDefinitionPredecessor {
                maximum_work_units, ..
            }
            | Self::WorkflowDefinitionCurrent {
                maximum_work_units, ..
            } => maximum_work_units.saturating_add(1),
            Self::WorkflowInstanceCapacity {
                maximum_instances, ..
            } => maximum_instances.saturating_mul(2).saturating_add(2),
            Self::IndexedEntitySelection {
                candidate_limit, ..
            } => candidate_limit.saturating_add(1),
            Self::Relation { .. } => usize::MAX,
            Self::Field { .. } | Self::AbsentField { .. } => 0,
        }
    }

    /// Whether the fact moved between the basis that read it and `snapshot`.
    /// The comparison reserves the least of its worst case and the meter's
    /// remaining work before it reads, reads within that reservation, and
    /// settles at what it spent. Indexed probes settle their actual debit on
    /// failure as well; other unanswered comparisons keep their reservation.
    /// A read the reservation cut short is the meter's stop.
    pub(in crate::domain_computation::primary_graph) fn source_currentness_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        // Decision facts are value observations, not native revisions.
        // Output lineage must first rebase them at the committed snapshot.
        if matches!(self, Self::Field { .. } | Self::AbsentField { .. }) {
            return Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable));
        }
        let most = self.most_comparison_work();
        let reserved_work = most.min(admission.remaining_work());
        let reserved =
            admission.reserve_external_work(u64::try_from(reserved_work).unwrap_or(u64::MAX))?;
        if matches!(self, Self::IndexedEntitySelection { .. }) {
            let mut remaining = reserved_work;
            let comparison = super::indexed_entity_selection::currentness_with_remaining(
                self,
                runtime,
                snapshot,
                &mut remaining,
            );
            reserved.settle(u64::try_from(reserved_work - remaining).unwrap_or(u64::MAX))?;
            return match comparison {
                Ok(equal) => Ok(Ok(FactMovement::from_equal(equal))),
                Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
                    if reserved_work < most =>
                {
                    let maximum = admission.charged_work().saturating_add(
                        u64::try_from(admission.remaining_work()).unwrap_or(u64::MAX),
                    );
                    Err(CompanionPreflightStop::WorkExhausted {
                        required: maximum.saturating_add(1),
                        maximum,
                    })
                }
                Err(failure) => Ok(Err(failure)),
            };
        }
        match self.comparison(runtime, snapshot, reserved_work) {
            Ok((equal, work)) => {
                reserved.settle(u64::try_from(work).unwrap_or(u64::MAX))?;
                Ok(Ok(FactMovement::from_equal(equal)))
            }
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) if reserved_work < most => {
                // The reservation is the meter's remaining work, and the
                // comparison needed more.
                let maximum = admission.charged_work();
                Err(CompanionPreflightStop::WorkExhausted {
                    required: maximum.saturating_add(1),
                    maximum,
                })
            }
            Err(failure) => Ok(Err(failure)),
        }
    }
}

#[cfg(test)]
impl WorthQueryApplicationObservedFact {
    /// The comparison on a meter of `maximum_work` alone, with the work it
    /// charged. A meter it runs out is the comparison's work failure.
    pub(in crate::domain_computation::primary_graph) fn source_currentness_within(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        maximum_work: usize,
    ) -> Result<(FactMovement, usize), WorthQuerySourceCurrentnessFailure> {
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: u64::try_from(maximum_work).unwrap_or(u64::MAX),
                maximum_preparation_bytes: 0,
            },
        );
        let movement = self
            .source_currentness_in(runtime, snapshot, &mut admission)
            .map_err(|_| WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)??;
        let work = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
        Ok((movement, work))
    }
}

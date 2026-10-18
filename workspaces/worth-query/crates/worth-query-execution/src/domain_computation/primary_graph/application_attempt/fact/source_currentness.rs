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
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Unavailable,
    OutputObservation(crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop),
}

impl WorthQueryApplicationObservedFact {
    /// The most comparing this fact can cost: its probe, and for a native
    /// probe that found nothing the liveness check that answers it
    /// (`unanswered_probe`). A relation fact records no bound of its own, so
    /// only the meter bounds it (`None`).
    fn most_comparison_work(&self) -> Option<usize> {
        Some(match self {
            Self::SourcePublication { .. } => return None,
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
            Self::Relation { .. } => return None,
            Self::Field { .. } | Self::AbsentField { .. } => 0,
        })
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        if !crate::domain_computation::execution_observation::is_installed() {
            return self.compare_source_currentness(runtime, execution, snapshot, admission);
        }
        super::super::super::fact_verification_observation::source_currentness();
        let before = admission.charged_work();
        let result = self.compare_source_currentness(runtime, execution, snapshot, admission);
        super::super::super::fact_verification_observation::source_currentness_work(
            admission
                .charged_work()
                .checked_sub(before)
                .expect("comparison retains its prior charges"),
        );
        result
    }

    fn compare_source_currentness(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Result<FactMovement, WorthQuerySourceCurrentnessFailure>, CompanionPreflightStop>
    {
        check_request(execution)?;
        // Decision facts are value observations, not native revisions.
        // Output lineage must first rebase them at the committed snapshot.
        if matches!(self, Self::Field { .. } | Self::AbsentField { .. }) {
            return Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable));
        }
        let most = self.most_comparison_work();
        let reserved_work = most.unwrap_or(usize::MAX).min(admission.remaining_work());
        let reserved = admission.reserve_external_work(
            u64::try_from(reserved_work)
                .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        if matches!(self, Self::IndexedEntitySelection { .. }) {
            let mut remaining = reserved_work;
            let comparison = super::indexed_entity_selection::currentness_with_remaining(
                self,
                runtime,
                execution,
                snapshot,
                &mut remaining,
            );
            reserved.settle(
                u64::try_from(reserved_work - remaining)
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
            return match comparison {
                Err(stop @ WorthQuerySourceCurrentnessFailure::OutputObservation(_)) => {
                    Ok(Err(stop))
                }
                Ok(equal) => Ok(Ok(FactMovement::from_equal(equal))),
                Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
                    comparison_work_failure(most, reserved_work, admission).map(Err)
                }
                Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
                    Err(CompanionPreflightStop::Execution(cause))
                }
                Err(WorthQuerySourceCurrentnessFailure::Unavailable) => {
                    Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable))
                }
            };
        }
        let comparison = self.comparison(runtime, execution, snapshot, reserved_work);
        match comparison {
            Err(stop @ WorthQuerySourceCurrentnessFailure::OutputObservation(_)) => Ok(Err(stop)),
            Ok((equal, work)) => {
                reserved.settle(
                    u64::try_from(work).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
                )?;
                Ok(Ok(FactMovement::from_equal(equal)))
            }
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded) => {
                comparison_work_failure(most, reserved_work, admission).map(Err)
            }
            Err(WorthQuerySourceCurrentnessFailure::Execution(cause)) => {
                Err(CompanionPreflightStop::Execution(cause))
            }
            Err(WorthQuerySourceCurrentnessFailure::Unavailable) => {
                Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable))
            }
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        maximum_work: usize,
    ) -> Result<(FactMovement, usize), WorthQuerySourceCurrentnessFailure> {
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: u64::try_from(maximum_work)
                    .map_err(|_| WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)?,
                maximum_preparation_bytes: 0,
            },
        );
        let movement = self
            .source_currentness_in(runtime, execution, snapshot, &mut admission)
            .map_err(|stop| match stop {
                CompanionPreflightStop::Execution(cause) => WorthQuerySourceCurrentnessFailure::Execution(cause),
                CompanionPreflightStop::Interrupted(event) => WorthQuerySourceCurrentnessFailure::Execution(match event.interruption() {
                    worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
                    worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed,
                }),
                CompanionPreflightStop::WorkExhausted { .. } | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. } | CompanionPreflightStop::WorkCounterOverflow => WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded,
                CompanionPreflightStop::TopologyPending | CompanionPreflightStop::SelectedSourceMismatch | CompanionPreflightStop::SelectedPositionUnavailable { .. } | CompanionPreflightStop::ForeignCell | CompanionPreflightStop::RegistrationChanged | CompanionPreflightStop::PreparationMemoryExhausted { .. } | CompanionPreflightStop::PreparationMemoryCounterOverflow | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. } => WorthQuerySourceCurrentnessFailure::Unavailable,
            })??;
        let work = usize::try_from(admission.charged_work())
            .map_err(|_| WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)?;
        Ok((movement, work))
    }
}

/// No fact-specific limit means a comparison overrun belongs to the meter,
/// even when its count bound reaches the machine word's maximum.
fn comparison_work_failure(
    most: Option<usize>,
    reserved: usize,
    admission: &InvalidationEditAdmission,
) -> Result<WorthQuerySourceCurrentnessFailure, CompanionPreflightStop> {
    if most.is_none_or(|most| reserved < most) {
        let maximum = admission
            .charged_work()
            .saturating_add(admission.remaining_work_amount());
        Err(CompanionPreflightStop::WorkExhaustedWithoutRequirement { maximum })
    } else {
        Ok(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
    }
}

fn check_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), CompanionPreflightStop> {
    if let Err(stop) = execution.consult() {
        if let worth_execution::WorkCeilingDenial::Stopped(stopped) = stop
            && let Ok(event) = worth_relational::facade::mvcc::RelationalInterruptionEvent::from_request_stop(
                stopped, worth_relational::facade::mvcc::RelationalInterruptionBoundary::ObservationAdmission,
            ) {
                return Err(CompanionPreflightStop::Interrupted(event));
            }
        return Err(CompanionPreflightStop::Execution(stop.into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::identity::{EntityId, KindId, PartitionId};

    #[test]
    fn relation_overrun_is_the_meter_stop_at_every_pointer_width() {
        // Identity is irrelevant to refusal classification; no runtime read is made.
        let fact = WorthQueryApplicationObservedFact::Relation {
            relation_kind: KindId::new(1),
            from: EntityId::new(PartitionId::main(), 0, 1),
            to: EntityId::new(PartitionId::main(), 1, 1),
            matching_relations: Vec::new(),
        };
        let reserved = usize::try_from(u32::MAX).unwrap();
        for (maximum, prior) in [
            (u64::from(u32::MAX), 0),
            (1_u64 << 40, (1_u64 << 40) - u64::from(u32::MAX)),
            (1_u64 << 40, 0),
        ] {
            let mut admission = InvalidationEditAdmission::new(
                worth_relational::facade::mvcc::CompanionPreflightBudget {
                    maximum_work_visits: maximum,
                    maximum_preparation_bytes: 0,
                },
            );
            admission.charge_external_work(prior).unwrap();
            {
                let _reservation = admission
                    .reserve_external_work(u64::from(u32::MAX))
                    .unwrap();
            }
            let paid = prior + u64::from(u32::MAX);
            assert_eq!(admission.charged_work(), paid);
            assert_eq!(admission.remaining_work_amount(), maximum - paid);
            if maximum > u64::from(u32::MAX) && prior == 0 {
                assert!(
                    maximum - paid > u64::from(u32::MAX),
                    "the unspent amount exceeds every 32-bit count bound"
                );
            }
            assert_eq!(
                comparison_work_failure(fact.most_comparison_work(), reserved, &admission),
                Err(CompanionPreflightStop::WorkExhaustedWithoutRequirement { maximum }),
                "a relation has no fact limit: its overrun has no invented requirement at every width"
            );
            assert_eq!(
                comparison_work_failure(Some(2), 2, &admission),
                Ok(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded),
                "a fact's own limit remains a fact refusal"
            );
        }
    }
}

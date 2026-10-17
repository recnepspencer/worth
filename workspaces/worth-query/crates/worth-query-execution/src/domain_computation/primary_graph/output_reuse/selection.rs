use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

use crate::domain_computation::primary_graph::{
    application_attempt::{
        Movement, WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure,
    },
    output_lineage::{
        cutoff_declines,
        invalidation::{FullVerificationReason, InvalidationEditAdmission},
        RecordedSettlementIdentity, SealedNativeOutputWitness,
    },
    SourceInvalidationOwner, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Internal choice; callers can only request an output through demand
/// admission. FreshRequired never grants authority to preserve an output.
pub(in crate::domain_computation::primary_graph) enum OutputDependencySelection {
    Reuse,
    FreshRequired,
}

pub(in crate::domain_computation::primary_graph) fn compare_retained_output_dependencies<
    F: FactSequence + ?Sized,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    identity_current: bool,
    facts: Option<&F>,
    admission: &mut InvalidationEditAdmission,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let facts = match reusable_facts(identity_current, facts) {
        Ok(facts) => facts,
        Err(fresh) => return Ok(fresh),
    };
    match first_moved_fact(execution, runtime, snapshot, facts.iter(), admission) {
        Ok(None) => Ok(OutputDependencySelection::Reuse),
        Ok(Some(_)) => Ok(OutputDependencySelection::FreshRequired),
        Err(stop) => Err(retained_fact_denial(
            stop,
            "retained comparison unavailable",
        )),
    }
}

pub(in crate::domain_computation::primary_graph) fn retained_fact_denial(
    stop: RetainedFactStop<'_>,
    unavailable: &str,
) -> WorthQueryOutputDemandDenial {
    match stop {
        RetainedFactStop::Work => work_denial(),
        RetainedFactStop::Memory => memory_denial(),
        RetainedFactStop::Execution(cause) => read_denial(cause),
        RetainedFactStop::Interrupted(event) => interruption_denial(event),
        RetainedFactStop::Refused(refusal) => WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            format!("{unavailable}: {refusal:?}"),
        ),
        RetainedFactStop::Unavailable(fact) => WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            fact.locator_identity(),
        ),
    }
}

/// Why a retained output's facts could not be compared.
pub(in crate::domain_computation::primary_graph) enum RetainedFactStop<'fact> {
    /// The request's meter, or the fact's own recorded bound, stopped it.
    Work,
    Memory,
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Interrupted(worth_relational::facade::mvcc::RelationalInterruptionEvent),
    Refused(CompanionPreflightStop),
    /// The snapshot cannot answer this fact.
    Unavailable(&'fact WorthQueryApplicationObservedFact),
}

/// The first of a retained output's facts that moved at `snapshot`, compared
/// in order and paid from the request's meter. `None` when every fact holds.
pub(in crate::domain_computation::primary_graph) fn first_moved_fact<'fact>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<&'fact WorthQueryApplicationObservedFact>, RetainedFactStop<'fact>> {
    for fact in facts {
        match fact.source_currentness_in(runtime, execution, snapshot, admission) {
            Ok(Ok(movement)) if movement.movement() == Movement::Moved => return Ok(Some(fact)),
            Ok(Ok(_)) => {}
            Ok(Err(WorthQuerySourceCurrentnessFailure::Execution(cause))) => {
                return Err(RetainedFactStop::Execution(cause))
            }
            Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable)) => {
                return Err(RetainedFactStop::Unavailable(fact));
            }
            Err(CompanionPreflightStop::Interrupted(event)) => {
                return Err(RetainedFactStop::Interrupted(event))
            }
            Ok(Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)) => {
                return Err(RetainedFactStop::Work)
            }
            Err(stop) => return Err(preflight_stop(stop)),
        }
    }
    Ok(None)
}

pub(in crate::domain_computation::primary_graph) fn preflight_stop(
    stop: CompanionPreflightStop,
) -> RetainedFactStop<'static> {
    match stop {
        CompanionPreflightStop::Execution(cause) => RetainedFactStop::Execution(cause),
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. } => {
            RetainedFactStop::Memory
        }
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. } => RetainedFactStop::Work,
        CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            RetainedFactStop::Execution(NativeReadStop::ChargedBytesOverflow)
        }
        CompanionPreflightStop::WorkCounterOverflow => {
            RetainedFactStop::Execution(NativeReadStop::WorkCounterOverflow)
        }
        CompanionPreflightStop::Interrupted(event) => RetainedFactStop::Interrupted(event),
        refusal @ (CompanionPreflightStop::TopologyPending
        | CompanionPreflightStop::SelectedSourceMismatch
        | CompanionPreflightStop::SelectedPositionUnavailable { .. }
        | CompanionPreflightStop::ForeignCell
        | CompanionPreflightStop::RegistrationChanged) => RetainedFactStop::Refused(refusal),
    }
}

pub(in crate::domain_computation::primary_graph) fn memory_denial() -> WorthQueryOutputDemandDenial
{
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "retained output comparison exceeded preparation memory",
    )
}

/// The other half of a retained output's canonical fact set: the performed
/// output itself. A role entity or aspect revision that moved since the
/// producer published selects producer execution over the live output, the
/// same verdict marking and consumed-output checks reach from this witness.
/// A row no sealed witness covers is decided by its source facts and by the
/// full verification its lineage row requires.
pub(in crate::domain_computation::primary_graph) fn compare_retained_output_witness(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    witness: Option<&SealedNativeOutputWitness>,
    admission: &mut InvalidationEditAdmission,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let Some(witness) = witness else {
        return Ok(OutputDependencySelection::Reuse);
    };
    let unchanged = owner_read(
        witness.unchanged_in(runtime, snapshot, admission),
        "retained output witness could not be compared",
    )?;
    Ok(if unchanged {
        OutputDependencySelection::Reuse
    } else {
        OutputDependencySelection::FreshRequired
    })
}

/// Whether the input cutoff verifies this row's settlement under the selected
/// source. A row it declines cannot be an exact selection: the cutoff would
/// run its producer again, so the live output selects the Preserve posture.
pub(in crate::domain_computation::primary_graph) fn retained_output_settlement_is_verified(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    requirement: Option<FullVerificationReason>,
    settlement: &RecordedSettlementIdentity,
    owner: &SourceInvalidationOwner,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    const UNAVAILABLE: &str = "retained output settlement could not be read";
    if cutoff_declines(requirement, None) {
        return Ok(false);
    }
    let selected = runtime
        .read_truth()
        .positioned_snapshot(snapshot)
        .map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                UNAVAILABLE,
            )
        })?;
    let current = owner_read(
        owner.currentness(&selected, settlement, admission),
        UNAVAILABLE,
    )?;
    Ok(!cutoff_declines(requirement, Some(&current)))
}

/// One owner read on the selection's meter: what it charged stays spent
/// whether or not it answered.
pub(in crate::domain_computation::primary_graph) fn owner_read<Answer>(
    answer: Result<Answer, CompanionPreflightStop>,
    unavailable: &'static str,
) -> Result<Answer, WorthQueryOutputDemandDenial> {
    answer.map_err(|stop| retained_fact_denial(preflight_stop(stop), unavailable))
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "output dependency comparison exceeded the admitted work budget",
    )
}

fn reusable_facts<F: FactSequence + ?Sized>(
    identity_current: bool,
    facts: Option<&F>,
) -> Result<&F, OutputDependencySelection> {
    if !identity_current {
        return Err(OutputDependencySelection::FreshRequired);
    }
    facts
        .filter(|facts| !facts.is_empty())
        .ok_or(OutputDependencySelection::FreshRequired)
}

pub(in crate::domain_computation::primary_graph) fn read_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as Execution;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::facade::primary_graph::relational_execution_kind(cause, None) {
        Ok(kind) => WorthQueryOutputDemandDenialKind::SourceQueryExecution(
            Execution::ExecutionRequest(kind),
        ),
        Err(Control::Cancelled) => WorthQueryOutputDemandDenialKind::Cancelled,
        Err(Control::TimedOut) => WorthQueryOutputDemandDenialKind::TimedOut,
    };
    WorthQueryOutputDemandDenial::new(kind, "retained output comparison")
}

pub(in crate::domain_computation::primary_graph) fn interruption_denial(
    event: worth_relational::facade::mvcc::RelationalInterruptionEvent,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::RelationalOperationInterruption;
    let kind = match event.interruption() {
        RelationalOperationInterruption::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
        RelationalOperationInterruption::TimedOut => WorthQueryOutputDemandDenialKind::TimedOut,
    };
    WorthQueryOutputDemandDenial::new(kind, "retained output comparison")
}

#[cfg(test)]
mod tests {
    use super::{reusable_facts, OutputDependencySelection};

    #[test]
    fn missing_dependency_facts_never_authorize_retained_output() {
        assert!(matches!(
            reusable_facts::<[super::WorthQueryApplicationObservedFact]>(true, None),
            Err(OutputDependencySelection::FreshRequired)
        ));
        assert!(matches!(
            reusable_facts(true, Some(&[])),
            Err(OutputDependencySelection::FreshRequired)
        ));
        assert!(matches!(
            reusable_facts::<[super::WorthQueryApplicationObservedFact]>(false, None),
            Err(OutputDependencySelection::FreshRequired)
        ));
    }
}

#[cfg(test)]
mod read_fold_tests {
    use super::*;
    #[test]
    fn read_fold_output_memory_stop_keeps_memory() {
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: 10,
                maximum_preparation_bytes: 0,
            },
        );
        let stop = admission.admit_read_scratch(1).unwrap_err();
        let denial = retained_fact_denial(preflight_stop(stop), "retained comparison unavailable");
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        );
    }
}

use super::*;
use crate::domain_computation::primary_graph::output_reuse;
pub(super) fn require_witnessed_output(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    witness: Option<
        &std::sync::Arc<
            std::sync::OnceLock<
                crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
            >,
        >,
    >,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    let witness = witness
        .and_then(|witness| witness.get())
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable))?;
    match witness.unchanged_in( relational, snapshot, admission) {
        Err(CompanionPreflightStop::Execution(cause)) => Err(output_reuse::read_denial(cause)),
        Ok(true) => Ok(()),
        Ok(false) => Err(denial(WorthQueryOutputDemandDenialKind::Superseded)),
        Err(
            CompanionPreflightStop::WorkExhausted { .. }
            | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
            ,
        ) => Err(denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)),
        Err(CompanionPreflightStop::Interrupted(event)) => {
            Err(output_reuse::interruption_denial(event))
        }
        Err(CompanionPreflightStop::WorkCounterOverflow) => Err(output_reuse::read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)),
        Err(CompanionPreflightStop::PreparationMemoryCounterOverflow | CompanionPreflightStop::RetainedCompanionCounterOverflow { .. }) => Err(output_reuse::read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow)),
        Err(CompanionPreflightStop::PreparationMemoryExhausted { .. } | CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }) => Err(output_reuse::memory_denial()),
        Err(CompanionPreflightStop::TopologyPending | CompanionPreflightStop::SelectedSourceMismatch | CompanionPreflightStop::SelectedPositionUnavailable { .. } | CompanionPreflightStop::ForeignCell | CompanionPreflightStop::RegistrationChanged) => Err(denial(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable)),
    }
}

pub(super) fn require_current_facts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match first_moved_fact(execution, relational, snapshot, facts.iter(), admission) {
        Ok(None) => Ok(()),
        Ok(Some(moved)) => Err(denial_subject(
            WorthQueryOutputDemandDenialKind::Superseded,
            moved.locator_identity(),
        )),
        Err(RetainedFactStop::Execution(cause)) => Err(output_reuse::read_denial(cause)),
        Err(RetainedFactStop::Interrupted(event)) => Err(output_reuse::interruption_denial(event)),
        Err(RetainedFactStop::Memory) => Err(output_reuse::memory_denial()),
        Err(RetainedFactStop::Work) => {
            Err(denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))
        }
        Err(RetainedFactStop::Refused(refusal)) => Err(denial_subject(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            format!("comparison refused: {refusal:?}"),
        )),
        Err(RetainedFactStop::Unavailable(fact)) => Err(denial_subject(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            fact.locator_identity(),
        )),
    }
}

fn denial_subject(
    kind: WorthQueryOutputDemandDenialKind,
    fact: String,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, format!("settled output fact is not current: {fact}"))
}

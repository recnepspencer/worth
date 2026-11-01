use super::RelationalStoreSection;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableRecoverySource, RelationalRecoveryStop,
};
use worth_store::physical_runtime::AnchoredRecordMembership;

#[derive(Debug)]
pub enum RelationalSectionReadmissionDenial {
    InitialSchemaRuntimeMismatch,
    RuntimeSharedBeforeReadmission,
    MalformedSection,
    MemberSelection(crate::RelationalStoreMemberSelectionDenial),
    Execution(worth_execution::WorkCeilingDenial),
    Payload(worth_execution::ExecutionRequestArrayDenial),
    Journal(crate::RelationalStoreJournalRefusal),
    Transitions(crate::RelationalStoreTransitionJournalRefusal),
    Native(RelationalRecoveryStop),
}

impl RelationalStoreSection {
    /// Readmit the section chosen by the application's single manifest and
    /// the two uncovered tails. No independent checkpoint anchor is consulted.
    pub fn readmit(
        &mut self,
        selected: Option<(&[u8], &AnchoredRecordMembership)>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalSectionReadmissionDenial> {
        let runtime = std::sync::Arc::get_mut(&mut self.runtime)
            .ok_or(RelationalSectionReadmissionDenial::RuntimeSharedBeforeReadmission)?;
        let (outcome, recovered) = recover_runtime(
            runtime,
            &self.cold_source,
            &self.serving,
            selected,
            execution,
        )?;
        self.recovered = Some(recovered);
        self.recovery_outcome = Some(outcome);
        Ok(())
    }
}

pub(super) fn recover_runtime(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    cold_source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    selected: Option<(&[u8], &AnchoredRecordMembership)>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        worth_relational::facade::durability::RecoveryOutcome,
        worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    ),
    RelationalSectionReadmissionDenial,
> {
    let mut admitted = None;
    let scope = execution.in_scope(|_| {
        admitted = Some(recover_in_scope(
            runtime,
            cold_source,
            serving,
            selected,
            execution,
        ))
    });
    match (scope, admitted) {
        (_, Some(Ok(completed))) => Ok(completed),
        (_, Some(Err(cause))) => Err(cause),
        (Err(cause), None) => Err(RelationalSectionReadmissionDenial::Execution(cause)),
        (Ok(()), None) => unreachable!("completed original-request native readmission"),
    }
}
fn recover_in_scope(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    cold_source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    selected: Option<(&[u8], &AnchoredRecordMembership)>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        worth_relational::facade::durability::RecoveryOutcome,
        worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    ),
    RelationalSectionReadmissionDenial,
> {
    use RelationalSectionReadmissionDenial as Denial;
    let checkpoint = selected
        .map(|(bytes, membership)| {
            let checkpoint = super::codec::decode(bytes, execution)?;
            cold_source
                .select_members(membership)
                .map_err(Denial::MemberSelection)?;
            Ok(checkpoint)
        })
        .transpose()?;
    let covered = checkpoint.as_ref().map_or(0, |section| section.covered().0);
    let transitions = checkpoint
        .as_ref()
        .map_or(0, |section| section.covered_transitions().0);
    let tail = crate::recovered_history::journal_tail(serving, covered).map_err(Denial::Journal)?;
    let transitions = crate::recovered_transitions::transition_tail(serving, transitions)
        .map_err(Denial::Transitions)?;
    let source =
        RelationalDurableRecoverySource::new(checkpoint, tail, transitions, cold_source.clone());
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .map_err(Denial::Native)?;
    runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .map_err(Denial::Native)
}

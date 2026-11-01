use super::RelationalStoreSection;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveredRelationalRuntimeAuthority, RecoveryOutcome, RelationalDurableCheckpointSource,
};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableRecoverySource, RelationalRecoveryStop,
};
use worth_store::physical_runtime::{AnchoredRecordMembership, AnchoredRecordReadMembership};

pub(super) struct ReadmittedRuntime {
    pub(super) outcome: RecoveryOutcome,
    pub(super) authority: RecoveredRelationalRuntimeAuthority,
    pub(super) selected_coverage: Option<[u64; 2]>,
}

#[derive(Debug)]
pub enum RelationalSectionReadmissionDenial {
    InitialSchemaRuntimeMismatch,
    RuntimeSharedBeforeReadmission,
    MalformedSection,
    CompleteNativeRequired,
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
        let checkpoint = prepare_ordinary_source(&self.cold_source, selected, execution)?;
        let admitted = recover_runtime(
            runtime,
            &self.cold_source,
            &self.serving,
            checkpoint,
            execution,
        )?;
        self.recovered = Some(admitted.authority);
        self.recovery_outcome = Some(admitted.outcome);
        Ok(())
    }

    /// The closed Home recipe supplies its one verified selected read. The
    /// returned coverage is descriptive and exists only after Native admission.
    pub fn readmit_home_from_read(
        &mut self,
        selected: Option<(
            worth_execution::ExecutionImmutableBytes,
            std::ops::Range<usize>,
            &AnchoredRecordReadMembership,
        )>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<[u64; 2]>, RelationalSectionReadmissionDenial> {
        let runtime = std::sync::Arc::get_mut(&mut self.runtime)
            .ok_or(RelationalSectionReadmissionDenial::RuntimeSharedBeforeReadmission)?;
        let checkpoint = prepare_home_source(&self.cold_source, selected, execution)?;
        let admitted = recover_runtime(
            runtime,
            &self.cold_source,
            &self.serving,
            checkpoint,
            execution,
        )?;
        self.recovered = Some(admitted.authority);
        self.recovery_outcome = Some(admitted.outcome);
        Ok(admitted.selected_coverage)
    }
}

pub(super) fn prepare_ordinary_source(
    cold_source: &crate::cold_source::StoreColdSource,
    selected: Option<(&[u8], &AnchoredRecordMembership)>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Option<RelationalDurableCheckpointSource>, RelationalSectionReadmissionDenial> {
    selected
        .map(|(bytes, membership)| {
            let checkpoint = super::codec::decode(bytes, execution)?;
            cold_source
                .select_members(membership)
                .map_err(RelationalSectionReadmissionDenial::MemberSelection)?;
            Ok(checkpoint)
        })
        .transpose()
}

pub(super) fn prepare_home_source(
    cold_source: &crate::cold_source::StoreColdSource,
    selected: Option<(
        worth_execution::ExecutionImmutableBytes,
        std::ops::Range<usize>,
        &AnchoredRecordReadMembership,
    )>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Option<RelationalDurableCheckpointSource>, RelationalSectionReadmissionDenial> {
    selected
        .map(|(_backing, _region, membership)| {
            cold_source
                .select_initial_members_from_read(membership, execution)
                .map_err(RelationalSectionReadmissionDenial::MemberSelection)?;
            // The qualified owner currently admits ordinary CHK17 only. A
            // coupled Home requires the real Complete source-admission door;
            // a description or format marker cannot stand in for that proof.
            Err(RelationalSectionReadmissionDenial::CompleteNativeRequired)
        })
        .transpose()
}

pub(super) fn recover_runtime(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    cold_source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    checkpoint: Option<RelationalDurableCheckpointSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedRuntime, RelationalSectionReadmissionDenial> {
    let mut admitted = None;
    let scope = execution.in_scope(|_| {
        admitted = Some(recover_in_scope(
            runtime,
            cold_source,
            serving,
            checkpoint,
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
    checkpoint: Option<RelationalDurableCheckpointSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedRuntime, RelationalSectionReadmissionDenial> {
    use RelationalSectionReadmissionDenial as Denial;
    let selected_coverage = checkpoint
        .as_ref()
        .map(|section| [section.covered().0, section.covered_transitions().0]);
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
    let (outcome, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .map_err(Denial::Native)?;
    Ok(ReadmittedRuntime {
        outcome,
        authority,
        selected_coverage,
    })
}

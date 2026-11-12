pub(super) mod fresh_source;
pub(super) mod prepared_source;
pub(super) mod source;

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
    FreshSourceAdmissionRequired,
    RecoveredSourceAdmissionRequired,
    RuntimeSharedBeforeReadmission,
    MalformedSection,
    PreparedHomeSourceRequired,
    HomePreparation(worth_relational::facade::durability::RelationalHomeRecoveryPreparationDenial),
    AdmissionImage(worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal),
    UnsupportedCompleteSectionVersion(
        worth_foundational::facade::BoundaryProtocolUnsupportedVersion,
    ),
    InitialInputs(worth_relational::facade::durability::RelationalInitialInputsRefusal),
    MemberSelection(crate::RelationalStoreMemberSelectionDenial),
    Execution(worth_execution::WorkCeilingDenial),
    Payload(worth_execution::ExecutionRequestArrayDenial),
    Journal(crate::RelationalStoreJournalRefusal),
    Transitions(crate::RelationalStoreTransitionJournalRefusal),
    Native(RelationalRecoveryStop),
    HomeProgramTransition(
        worth_relational::facade::durability::RelationalHomeProgramTransitionSourceRefusal,
    ),
    FreshSource(worth_relational::facade::durability::RelationalFreshSourceAdmissionStop),
    OwnedSource(worth_relational::facade::durability::RelationalDurableOwnedReadRefusal),
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
        self.source = Some(source::ReadmittedRelationalSource::Recovered {
            authority: admitted.authority,
            outcome: admitted.outcome,
        });
        Ok(())
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
            let original = crate::cold_source::selected_replacement::read_expected(
                cold_source,
                membership.version(),
                execution,
            )
            .map_err(RelationalSectionReadmissionDenial::MemberSelection)?;
            cold_source
                .select_initial_members_from_read(&original, execution)
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
) -> Result<
    Option<super::admission_section::DecodedHomeAdmission>,
    RelationalSectionReadmissionDenial,
> {
    selected
        .map(|(backing, region, membership)| {
            cold_source
                .select_initial_members_from_read(membership, execution)
                .map_err(RelationalSectionReadmissionDenial::MemberSelection)?;
            let version = membership.membership().version();
            if version.payload_bytes() != backing.len() as u64
                || super::admission_section::digest(backing.bytes(), execution)
                    .map_err(RelationalSectionReadmissionDenial::AdmissionImage)? != version.payload_sha256()
            {
                return Err(RelationalSectionReadmissionDenial::AdmissionImage(
                    worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal::BindingMismatch(
                        worth_relational::facade::durability::CheckpointAdmissionBindingMismatch::Source,
                    ),
                ));
            }
            super::admission_section::decode(backing, region, execution)
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
    _serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    checkpoint: Option<RelationalDurableCheckpointSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedRuntime, RelationalSectionReadmissionDenial> {
    use RelationalSectionReadmissionDenial as Denial;
    let selected_coverage = checkpoint
        .as_ref()
        .map(|section| [section.covered().0, section.covered_transitions().0]);
    let plan = prepare_source_plan(runtime, cold_source, checkpoint, execution)?;
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

pub(super) fn prepare_source_plan(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    cold_source: &std::sync::Arc<crate::cold_source::StoreColdSource>,
    checkpoint: Option<RelationalDurableCheckpointSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<worth_relational::facade::durability::RecoveryPlan, RelationalSectionReadmissionDenial>
{
    use RelationalSectionReadmissionDenial as Denial;
    let original: std::sync::Arc<
        dyn worth_relational::facade::durability::RelationalDurableColdSource,
    > = cold_source.clone();
    let protection =
        worth_relational::facade::durability::RelationalDurableReadProtection::capture(
            &original, execution,
        )
        .map_err(Denial::OwnedSource)?;
    let source = RelationalDurableRecoverySource::from_protected_source(
        execution,
        checkpoint,
        original,
        &protection,
    )
    .map_err(Denial::OwnedSource)?;
    runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .map_err(Denial::Native)
}

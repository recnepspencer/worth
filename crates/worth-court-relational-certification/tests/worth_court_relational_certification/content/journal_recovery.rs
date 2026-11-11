//! Decode real journal bytes before replay, never a retained in-memory envelope.
use super::{
    original,
    recovery::publish,
    transaction::{fixture, FORMAT},
};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
struct NoBacking;
impl RelationalDurableColdSource for NoBacking {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if members.is_empty() {
            Ok(RelationalDurableMemberCustody::new(()))
        } else {
            Err(Cause::MissingMember)
        }
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::MissingMember)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

pub(super) fn assert_retained_selection_journal_refuses(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) {
    let artifacts = runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .map(|artifact| {
            RelationalCanonicalCommitArtifact::from_untrusted_bytes(
                artifact.unwrap().as_bytes().to_vec(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(artifacts.len(), 3);
    let (mut target, _) = fixture();
    let before = target
        .observe_branch(&target.main_branch_identity())
        .unwrap()
        .1;
    let plan = target
        .durability()
        .recovery_plan_from_source(
            execution,
            RelationalDurableRecoverySource::new(None, artifacts, vec![], Arc::new(NoBacking)),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let refusal = target
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    assert!(
        matches!(refusal, RelationalRecoveryStop::Durability(ref error) if error.class == RecoveryFailureClass::UnsupportedRetainedEntitySelectionReplay),
        "{refusal:?}"
    );
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}

#[test]
fn content_decoded_journal_reconstructs_the_native_owner_without_a_checkpoint() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (id, entity) = publish(&runtime, &execution, &contract);
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    let transport =
        RelationalCanonicalCommitArtifact::from_untrusted_bytes(artifact.as_bytes().to_vec())
            .unwrap();
    let (mut target, _) = fixture();
    let source =
        RelationalDurableRecoverySource::new(None, vec![transport], vec![], Arc::new(NoBacking));
    let plan = target
        .durability()
        .recovery_plan_from_source(
            &execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    target
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap();
    super::selected_value::assert_selected(&target, &execution, entity, id);
    let (_, basis) = target
        .observe_branch(&target.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.read_content(&execution, id, FORMAT).unwrap().bytes(),
        super::expected::FRAME
    );
}

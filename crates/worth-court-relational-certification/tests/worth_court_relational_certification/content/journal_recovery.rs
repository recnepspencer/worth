//! Decode real journal bytes before replay, never a retained in-memory envelope.
use super::{
    checkpoint_corruption::value_end,
    original,
    recovery::publish,
    transaction::{fixture, FORMAT},
};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{
    content::RelationalContentDenial, durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
struct NoBacking;

#[test]
fn content_controlled_journal_seed_admits_the_canonical_omitted_history_tail() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish(&runtime, &execution, &contract);
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    let mut bytes = artifact.as_bytes().to_vec();
    let key = b"\xa9canonical";
    let map = bytes
        .windows(key.len())
        .position(|word| word == key)
        .unwrap()
        + key.len();
    // Independent MessagePack map framing; remove this optional history field
    // and repair the enclosing digest, never invoke the Native serializer.
    assert_eq!(bytes[map], 0xde);
    let count = u16::from_be_bytes(bytes[map + 1..map + 3].try_into().unwrap());
    let mut cursor = map + 3;
    let mut removed = false;
    for _ in 0..count {
        let value = value_end(&bytes, cursor);
        let end = value_end(&bytes, value);
        if &bytes[cursor..value] == b"\xb5authored_index_writes" {
            bytes.splice(cursor..end, []);
            bytes[map + 1..map + 3].copy_from_slice(&(count - 1).to_be_bytes());
            removed = true;
            break;
        }
        cursor = end;
    }
    assert!(removed, "the fixture originally carries authored history");
    crate::canonical_commit_frame::reseal(&mut bytes);
    // Successful admission includes Native's independent exact canonical
    // re-emission check. Missing history remains its supported unavailable tail.
    RelationalCanonicalCommitArtifact::readmit(&bytes, &execution).unwrap();
}
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
        .canonical_commit_artifacts(&execution)
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

#[test]
fn content_decoded_journal_omission_refuses_before_selecting_a_root() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish(&runtime, &execution, &contract);
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    let mut bytes = artifact.as_bytes().to_vec();
    let key = b"\xa7content\x81\xa7records";
    let start = bytes
        .windows(key.len())
        .position(|word| word == key)
        .unwrap()
        + key.len();
    assert_eq!(bytes[start], 0x91);
    let end = value_end(&bytes, start);
    bytes.splice(start..end, [0x90]);
    assert!(!bytes
        .windows(super::expected::FRAME.len())
        .any(|word| word == super::expected::FRAME));
    crate::canonical_commit_frame::reseal(&mut bytes);
    let transport = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes).unwrap();
    // Valid artifact data alone grants neither a frame resolver nor publication.
    RelationalCanonicalCommitArtifact::readmit(transport.as_bytes(), &execution).unwrap();
    let (mut target, _) = fixture();
    let before = target
        .observe_branch(&target.main_branch_identity())
        .unwrap()
        .0;
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
    let refusal = target
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalRecoveryStop::Content(RelationalContentDenial::MissingContent)
        ),
        "{refusal:?}"
    );
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}

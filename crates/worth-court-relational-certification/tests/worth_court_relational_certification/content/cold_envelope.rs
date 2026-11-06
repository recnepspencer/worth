//! Non-head canonical bodies reach content admission only on historical demand.
use super::{recovery::publish, transaction::fixture};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    content::RelationalContentRejection, durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause, transactions::RelationalExecutionDenialCause,
};

mod source;
pub(super) use source::Source;

fn reopen(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    manifest: Vec<u8>,
    source: Arc<Source>,
) -> worth_relational::facade::runtime::RelationalRuntime {
    reopen_with_tail(execution, checkpoint, manifest, source, vec![])
}

pub(super) fn reopen_with_tail(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    manifest: Vec<u8>,
    source: Arc<Source>,
    tail: Vec<RelationalCanonicalCommitArtifact>,
) -> worth_relational::facade::runtime::RelationalRuntime {
    let expected_replays = tail.len();
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(manifest),
        )),
        tail,
        vec![],
        source.clone(),
    );
    let (mut target, _) = fixture();
    let plan = target
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let recovered = target
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(recovered.coverage.replayed_tail_commits, expected_replays);
    target
}

#[test]
fn content_cold_envelope_preserves_a_literal_version_refusal_without_poisoning_history() {
    let cancellation = CancellationSource::new();
    let budget = SerialMemoryBudget::new(None);
    let original = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
    publish(&runtime, &execution, &contract);
    let older = runtime.history().historical_latest_commit().unwrap();
    let older_position = runtime.history().latest_patch_stream_position().unwrap();
    publish(&runtime, &execution, &contract);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let mut backing = Source::capture(&runtime, &execution, &checkpoint, older_position, None);
    let (old_member, mut bytes) = checkpoint
        .members()
        .iter()
        .find_map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes }
                if bytes.starts_with(b"WRTHRCCA")
                    && u64::from_le_bytes(bytes[10..18].try_into().unwrap())
                        == older_position.0 =>
            {
                Some((*locator, bytes.to_vec()))
            }
            RelationalCheckpointMember::New { .. } | RelationalCheckpointMember::Carried { .. } => {
                None
            }
        })
        .expect("the actual checkpoint owns the older canonical member");
    assert_eq!(old_member.sha256, <[u8; 32]>::from(Sha256::digest(&bytes)));
    let previous_digest: [u8; 32] = bytes[26..58].try_into().unwrap();
    let at = bytes
        .windows(super::expected::FRAME.len())
        .position(|word| word == super::expected::FRAME)
        .unwrap();
    bytes[at + 8..at + 10].copy_from_slice(&2_u16.to_le_bytes());
    // Independent physical framing repair reaches content's version check.
    // No canonical/root/field authority is constructed from these bytes.
    crate::canonical_commit_frame::reseal(&mut bytes);
    let digest = bytes[26..58].try_into().unwrap();
    let new_member = RelationalDurableMemberLocator {
        sha256: Sha256::digest(&bytes).into(),
        length: bytes.len() as u64,
    };
    backing.insert_selected_frame(&execution, new_member, &bytes);
    let physical = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
    assert_eq!(physical.position(), older_position);
    let mut manifest = checkpoint.bytes().to_vec();
    replace_index_digest(&mut manifest, previous_digest, digest);
    replace_member_locator(&mut manifest, old_member, new_member);
    let source = Arc::new(backing);
    let target = reopen(&execution, &checkpoint, manifest, source.clone());
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        0,
        "the non-head body is genuinely cold"
    );
    let (_, selected) = target
        .observe_branch(&target.main_branch_identity())
        .unwrap();
    for reads in 1..=2 {
        let refusal = target
            .history()
            .historical_committed_version(&execution, older.version_id)
            .unwrap_err();
        assert_eq!(
            refusal,
            RelationalExecutionDenialCause::Content(RelationalContentRejection::UnsupportedVersion)
        );
        assert_eq!(
            source.reads.load(Ordering::SeqCst),
            reads,
            "semantic eligibility refusal cannot poison the physical cell"
        );
        assert_eq!(
            target
                .observe_branch(&target.main_branch_identity())
                .unwrap()
                .1
                .descriptor(),
            selected.descriptor()
        );
    }
}

// A decoded historical body cannot certify field-content absence.
#[test]
fn content_cold_history_is_unavailable_while_a_genuine_dangling_reference_is_missing() {
    use worth_foundational::facade::ContentRefId;
    use worth_relational::facade::{
        content::RelationalContentDenial as Denial,
        mvcc::RelationalTransactionIntent,
        transactions::{ConflictClass, TransactionCommitError},
    };
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    // The older body is an actual Journal tail, not a selected member.
    let (identity, entity) = publish(&runtime, &execution, &contract);
    let older = runtime.history().latest_patch_stream_position().unwrap();
    publish(&runtime, &execution, &contract);
    let tail = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(Result::unwrap)
        .collect();
    let source = Arc::new(Source::capture(
        &runtime,
        &execution,
        &checkpoint,
        older,
        None,
    ));
    let target = reopen_with_tail(
        &execution,
        &checkpoint,
        checkpoint.bytes().to_vec(),
        source.clone(),
        tail,
    );
    drop(runtime);
    drop(checkpoint);
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        0,
        "the historical body starts cold"
    );
    let artifact = target
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(artifact.position(), older);
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        1,
        "a genuine cold journal demand ran"
    );
    let decoded =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), &execution).unwrap();
    assert_eq!(
        decoded
            .envelope()
            .content_for_field(
                &execution,
                entity,
                "test.entity",
                &super::historical_custody::locator("left"),
            )
            .unwrap_err(),
        Denial::HistoryNotResident,
    );
    let (_, basis) = target
        .observe_branch(&target.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis
            .read_content(&execution, identity, super::transaction::FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
    let mut transaction = target
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            super::transaction::batch(&contract, ContentRefId([0xff; 32])),
            execution.allocation_policy(),
        )
        .unwrap();
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(&execution, &target).unwrap_err()
    else {
        panic!("a genuine dangling field must refuse before publication")
    };
    assert_eq!(
        error.class,
        ConflictClass::ContentAdmission {
            denial: Denial::MissingContent
        }
    );
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}

// Test-side MessagePack framing of the specific index field, not its decoder.
fn replace_index_digest(bytes: &mut Vec<u8>, old: [u8; 32], new: [u8; 32]) {
    fn encoded(digest: [u8; 32]) -> Vec<u8> {
        let mut word = b"\xaepayload_digest\xdc\x00\x20".to_vec();
        for byte in digest {
            if byte >= 128 {
                word.push(0xcc);
            }
            word.push(byte);
        }
        word
    }
    let old = encoded(old);
    let positions: Vec<_> = bytes
        .windows(old.len())
        .enumerate()
        .filter_map(|(at, word)| (word == old).then_some(at))
        .collect();
    let [at] = positions.as_slice() else {
        panic!("one original canonical index digest, got {positions:?}")
    };
    bytes.splice(*at..*at + old.len(), encoded(new));
}

// Rewrite only this physical DATA locator, leaving all other wire values intact.
fn replace_member_locator(
    bytes: &mut Vec<u8>,
    old: RelationalDurableMemberLocator,
    new: RelationalDurableMemberLocator,
) {
    let old = rmp_serde::to_vec_named(&old).unwrap();
    let new = rmp_serde::to_vec_named(&new).unwrap();
    let positions: Vec<_> = bytes
        .windows(old.len())
        .enumerate()
        .filter_map(|(at, word)| (word == old).then_some(at))
        .collect();
    let [at] = positions.as_slice() else {
        panic!("one selected canonical locator, got {positions:?}")
    };
    bytes.splice(*at..*at + old.len(), new);
}

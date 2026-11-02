//! A non-head journal body reaches content admission only on historical demand.
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

#[derive(Debug)]
struct Source {
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    older: PatchStreamPosition,
    reads: AtomicUsize,
}
impl RelationalDurableColdSource for Source {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        bounded(
            self.members
                .get(&locator)
                .ok_or(RelationalStorageUnavailableCause::MissingMember)?,
            limit,
        )
    }
    fn read_commit(
        &self,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        if position == self.older {
            self.reads.fetch_add(1, Ordering::SeqCst);
        }
        bounded(
            self.commits
                .get(&position)
                .ok_or(RelationalStorageUnavailableCause::SourceUnavailable)?,
            limit,
        )
    }
}
fn bounded(bytes: &[u8], limit: u64) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
    if bytes.len() as u64 > limit {
        return Err(RelationalStorageUnavailableCause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
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
    let members = checkpoint
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
            _ => panic!("initial capture owns its members"),
        })
        .collect();
    let mut commits: BTreeMap<_, _> = runtime
        .durability()
        .canonical_commit_artifacts(execution.allocation_policy())
        .map(|artifact| {
            let artifact = artifact.unwrap();
            (artifact.position(), artifact.as_bytes().to_vec())
        })
        .collect();
    let bytes = commits.get_mut(&older_position).unwrap();
    let previous_digest: [u8; 32] = bytes[26..58].try_into().unwrap();
    let at = bytes
        .windows(super::expected::FRAME.len())
        .position(|word| word == super::expected::FRAME)
        .unwrap();
    bytes[at + 8..at + 10].copy_from_slice(&2_u16.to_le_bytes());
    // Independent physical framing repair reaches content's version check.
    // No canonical/root/field authority is constructed from these bytes.
    let digest: [u8; 32] = Sha256::digest(&bytes[66..]).into();
    bytes[26..58].copy_from_slice(&digest);
    let physical = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
    assert_eq!(physical.position(), older_position);
    let mut manifest = checkpoint.bytes().to_vec();
    replace_index_digest(&mut manifest, previous_digest, digest);
    let source = Arc::new(Source {
        members,
        commits,
        older: older_position,
        reads: AtomicUsize::new(0),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(manifest),
        )),
        vec![],
        vec![],
        source.clone(),
    );
    let (mut target, _) = fixture();
    let plan = target
        .durability()
        .recovery_plan_from_source(
            &execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    target
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap();
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
        panic!("one old journal index digest, got {positions:?}")
    };
    bytes.splice(*at..*at + old.len(), encoded(new));
}

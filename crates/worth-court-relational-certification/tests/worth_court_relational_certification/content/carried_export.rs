//! Read current truth first, then fault only the genuinely cold older journal body.
use super::{recovery::publish, transaction::fixture};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition, runtime::RelationalRuntime,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
struct Source {
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    fault: AtomicUsize,
    reads: AtomicUsize,
}
impl RelationalDurableColdSource for Source {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let bytes = self.members.get(&locator).ok_or(Cause::MissingMember)?;
        bounded(bytes, limit)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        let mut bytes = bounded(
            self.commits
                .get(&position)
                .ok_or(Cause::SourceUnavailable)?,
            limit,
        )?;
        if position == PatchStreamPosition(1) {
            self.reads.fetch_add(1, Ordering::SeqCst);
            match self.fault.load(Ordering::SeqCst) {
                0 => {}
                1 => bytes[8..10].copy_from_slice(&2_u16.to_le_bytes()),
                2 => bytes[58..66].copy_from_slice(&(256_u64 * 1024 * 1024 - 65).to_le_bytes()),
                3 => *bytes.last_mut().unwrap() ^= 1,
                value => panic!("unrecognized court fault {value}"),
            }
        }
        Ok(bytes)
    }
}
fn bounded(bytes: &[u8], limit: u64) -> Result<Vec<u8>, Cause> {
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
}
fn cold_runtime(execution: &ExecutionRequest<'_, '_>) -> (RelationalRuntime, Arc<Source>) {
    let (runtime, contract) = fixture();
    publish(&runtime, execution, &contract);
    publish(&runtime, execution, &contract);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
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
            _ => panic!("initial capture owns members"),
        })
        .collect();
    let commits = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(|artifact| {
            let artifact = artifact.unwrap();
            (artifact.position(), artifact.as_bytes().to_vec())
        })
        .collect();
    let source = Arc::new(Source {
        members,
        commits,
        fault: AtomicUsize::new(0),
        reads: AtomicUsize::new(0),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        )),
        vec![],
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
    target
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        0,
        "the older journal remains genuinely cold"
    );
    (target, source)
}

#[test]
fn content_carried_export_preserves_version_and_size_refusals_across_repeated_calls() {
    for fault in [1, 2] {
        let cancel = CancellationSource::new();
        let budget = SerialMemoryBudget::new(None);
        let request = SerialRequest::from_memory(budget, cancel.token(), None);
        let execution = ExecutionRequest::serial(&request);
        let (target, source) = cold_runtime(&execution);
        let before = target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .0;
        source.fault.store(fault, Ordering::SeqCst);
        for _ in 0..2 {
            let refusal = target
                .durability_authority()
                .native_checkpoint(&execution, execution.allocation_policy())
                .unwrap_err();
            let RelationalNativeCheckpointCaptureDenial::ArtifactFrame(frame) = refusal else {
                panic!("{refusal:?}");
            };
            match fault {
                1 => assert_eq!(
                    frame,
                    RelationalCanonicalCommitArtifactFrameRefusal::UnsupportedVersion(
                        worth_foundational::BoundaryProtocolCompatibilityWindow::inclusive(
                            worth_foundational::BoundaryProtocolVersion::new(
                                RelationalCanonicalCommitArtifact::FORMAT_VERSION as u32
                            ),
                            worth_foundational::BoundaryProtocolVersion::new(
                                RelationalCanonicalCommitArtifact::FORMAT_VERSION as u32
                            ),
                        )
                        .admit(worth_foundational::BoundaryProtocolVersion::new(2))
                        .unwrap_err(),
                    )
                ),
                2 => assert_eq!(
                    frame,
                    RelationalCanonicalCommitArtifactFrameRefusal::SizeLimit {
                        declared_payload: 256 * 1024 * 1024 - 65,
                        maximum_frame: 256 * 1024 * 1024
                    }
                ),
                value => panic!("unexpected fault {value}"),
            }
            assert_eq!(
                target
                    .observe_branch(&target.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        }
        assert_eq!(
            source.reads.load(Ordering::SeqCst),
            2,
            "eligibility refusal must not poison the backing as corrupt"
        );
    }
}

#[test]
fn content_carried_export_actual_digest_damage_becomes_a_sticky_typed_refusal() {
    let cancel = CancellationSource::new();
    let request = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let (target, source) = cold_runtime(&execution);
    source.fault.store(3, Ordering::SeqCst);
    let refusal = target
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalNativeCheckpointCaptureDenial::ArtifactFrame(
                RelationalCanonicalCommitArtifactFrameRefusal::DigestMismatch { .. }
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    let refusal = target
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap_err();
    assert!(
        matches!(refusal, RelationalNativeCheckpointCaptureDenial::Durability(ref error) if error.class == RecoveryFailureClass::ColdStorageUnavailable),
        "{refusal:?}"
    );
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        1,
        "proved damage stays sticky"
    );
}

#[test]
fn content_carried_export_reserves_the_certified_read_bound_before_the_port() {
    let cancel = CancellationSource::new();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(512 * 1024 * 1024)),
        cancel.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let (target, source) = cold_runtime(&execution);
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let limited = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let limited = ExecutionRequest::serial(&limited);
    let refusal = target
        .durability_authority()
        .native_checkpoint(&limited, limited.allocation_policy())
        .unwrap_err();
    assert!(matches!(refusal, RelationalNativeCheckpointCaptureDenial::Request(worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted { requested: 268_435_456, .. })), "{refusal:?}");
    assert_eq!(source.reads.load(Ordering::SeqCst), 0);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn content_carried_export_reconstructs_exact_bytes_through_the_existing_checkpoint_grammar() {
    use sha2::{Digest, Sha256};
    let cancel = CancellationSource::new();
    let request = SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let (target, source) = cold_runtime(&execution);
    let checkpoint = target
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    let (mut restored, _) = fixture();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    let mut independent = Sha256::new();
    independent.update(b"worth.native.content.v1\0");
    independent.update(super::expected::FRAME);
    let identity = worth_foundational::facade::ContentRefId(independent.finalize().into());
    let (_, basis) = restored
        .observe_branch(&restored.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis
            .read_content(&execution, identity, super::transaction::FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
}

//! A capture cannot replace canonical adjacency authority by rehashing leaves.
use super::*;
use crate::durability::data::*;
use crate::durability::medium::*;
use crate::identity::data::PartitionId;
use crate::tests::support::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[test]
fn checkpoint_readmission_refuses_retired_integer_array_digest_before_restore() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    create_entity(&execution, &runtime, "binary-digest-checkpoint");
    let native = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    decode_checkpoint(&execution, native.bytes()).unwrap();
    let wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    let digest = wire.manifest.mirror[0].kind_authority.head;
    let mut current = rmp_serde::to_vec("head").unwrap();
    current.extend(rmp_serde::to_vec(&digest).unwrap());
    let mut retired = rmp_serde::to_vec("head").unwrap();
    retired.extend(rmp_serde::to_vec(digest.bytes()).unwrap());
    assert_ne!(current, retired);
    // Rewrite only the representation of this actual captured digest. Every
    // semantic value and every other field keeps the owning producer's bytes.
    let mut bytes = Vec::new();
    let mut remaining = native.bytes();
    let mut replacements = 0;
    while let Some(at) = remaining
        .windows(current.len())
        .position(|part| part == current)
    {
        bytes.extend_from_slice(&remaining[..at]);
        bytes.extend_from_slice(&retired);
        remaining = &remaining[at + current.len()..];
        replacements += 1;
    }
    bytes.extend_from_slice(remaining);
    assert!(replacements > 0);
    let stop = decode_checkpoint(&execution, &bytes).unwrap_err();
    assert_eq!(
        expect_durability_failure(stop).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
}

#[test]
fn forged_cross_region_kind_commitment_is_refused_before_member_reads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let source = create_entity_in_partition(&execution, &runtime, "source", PartitionId(1));
    let target = create_entity_in_partition(&execution, &runtime, "target", PartitionId(2));
    create_relation_in_partition(
        &execution,
        &runtime,
        source,
        target,
        "cross-region",
        PartitionId(3),
    );
    let native = runtime
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
    let mut wire: checkpoint_wire::CheckpointWire<'_> =
        rmp_serde::from_slice(native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap()).unwrap();
    // Keep every copy of this leaf consistent; only canonical root authority
    // can reject the forgery. No member is needed to perform that check.
    let mut manifest = wire.manifest.ordinary_owned_draft();
    let mut roots = Vec::with_capacity(manifest.roots.len());
    for original in &manifest.roots {
        let mut regions = original.regions.clone();
        for leaf in &mut regions {
            if leaf.partition_id == PartitionId(1) {
                let mut altered_head = *leaf.kind_authority.head.bytes();
                altered_head[0] ^= 1;
                leaf.kind_authority.head = worth_foundational::CanonicalDigestId::new(altered_head);
            }
        }
        roots.push(RelationalCheckpointRoot::from_ordinary_parts(
            original.commit_id,
            original.recorded_components.clone(),
            original.schema_carrier_digest,
            regions,
            original.content.clone(),
        ));
    }
    for leaf in &mut manifest.mirror {
        if leaf.partition_id == PartitionId(1) {
            let mut altered_head = *leaf.kind_authority.head.bytes();
            altered_head[0] ^= 1;
            leaf.kind_authority.head = worth_foundational::CanonicalDigestId::new(altered_head);
        }
    }
    manifest.roots = CheckpointRoots::from_ordinary(roots);
    wire.manifest = manifest.seal_ordinary();
    let mut bytes = CHECKPOINT_MAGIC.to_vec();
    bytes.extend(rmp_serde::to_vec_named(&wire).unwrap());
    let source = Arc::new(ForgedKindSource {
        members: member_source::InlineCheckpointSource {
            members: {
                let mut members = worth_execution::ExecutionOrderedMap::default();
                for member in native.members() {
                    let RelationalCheckpointMember::New { locator, bytes } = member else {
                        panic!("first capture owns its members")
                    };
                    members
                        .insert_cloned_native_for_request(locator, bytes, execution)
                        .unwrap();
                }
                members
            },
        },
        journal: attached_durable_medium(&runtime).expect("fixture owns its actual durable medium"),
        reads: AtomicUsize::new(0),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut receiving = durable_runtime_with_test_schema();
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            &execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let failure = receiving
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap_err();
    assert_eq!(source.reads.load(Ordering::SeqCst), 0);
    assert!(matches!(failure, RelationalRecoveryStop::Durability(error)
        if error.class == RecoveryFailureClass::CorruptCheckpoint && error.detail.contains("StorageContentMismatch")));
}

#[derive(Debug)]
struct ForgedKindSource {
    members: member_source::InlineCheckpointSource,
    journal: Arc<dyn RelationalDurableColdSource>,
    reads: AtomicUsize,
}
impl RelationalDurableColdSource for ForgedKindSource {
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (RelationalDurableReadBounds, RelationalDurableReadPayload),
        RelationalDurableOwnedReadRefusal,
    > {
        self.members.checkpoint_read_protection(request)
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        self.members.checkpoint_selected_read_protection(request)
    }
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        self.members
            .retain_members_for_request(request, members, issuer)
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.members
            .read_member_owned(request, original, locator, limit)
    }
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<
        RelationalDurableMemberCustody,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        self.members.retain_members(members)
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.members.read_member(locator, limit)
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        self.journal.read_commit(position, limit)
    }
}

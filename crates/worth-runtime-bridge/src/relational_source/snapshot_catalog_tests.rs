use crate::facade::{
    CommittedPatchSource, RelationalBridgeRecordIdentityParts, RelationalCommittedPatchRequest,
    SnapshotReadContract, SnapshotReadRecord, SnapshotReadRequest, SnapshotReadSource,
    TruthCommitIdentity, TruthSnapshotIdentity,
};
use crate::relational_source::relational_test_support::{
    exact_change, published_patch, WireOperation,
};
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    AuthoritativeAspectChangeKind, FieldKey, ScalarAspectType,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, PartitionId, VersionId};
use worth_relational::facade::publication::PatchDetail;
use worth_relational::facade::publication::{
    PatchOrdering, PatchPublicationMode, PatchStreamPosition, PublishedAuthoritativePatchEnvelope,
    PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};

use super::{PublicationBridgeCatalog, PublicationBridgeSnapshot};

#[test]
fn publication_bridge_catalog_exposes_committed_patch_and_snapshot() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let catalog = PublicationBridgeCatalog::default();
    catalog
        .register_patch(
            CommitId(7),
            &BranchId("main".to_string()),
            TruthSnapshotIdentity::from_relational_snapshot(
                crate::facade::RelationalBridgeSnapshotIdentityParts::new(7, 7),
            ),
            &PublishedAuthoritativePatchEnvelope {
                ordering: PatchOrdering::CanonicalCommitOrder,
                publication_mode: PatchPublicationMode::CommitNative,
                position: PatchStreamPosition(11),
                authoritative_record_patches: vec![PublishedAuthoritativeRecordPatch {
                    target: worth_relational::facade::transactions::RecordRef::Entity(
                        EntityId::new(PartitionId::main(), 4, 1),
                    ),
                    structural_change: RecordStructuralChange::Updated,
                    authoritative_patch: published_patch(vec![WireOperation::WholeAspectClear {
                        aspect_key: AspectKey::new("profile.name").unwrap(),
                        aspect_identity: AspectIdentity(1),
                        contract_revision: AspectContractRevision(1),
                        binding: AspectBinding::EntityField {
                            field: FieldKey::new("name").unwrap(),
                        },
                    }]),
                    semantic_changes: vec![exact_change(
                        AspectKey::new("profile.name").unwrap(),
                        AspectIdentity(1),
                        AspectContractRevision(1),
                        AspectBinding::EntityField {
                            field: FieldKey::new("name").unwrap(),
                        },
                        AuthoritativeAspectChangeKind::WholeAspectClear,
                        None,
                    )],
                    contains_opaque_aspect: false,
                    detail: PatchDetail::DenseBitset(vec![1]),
                }],
            },
        )
        .expect("valid publication patch should register");
    let snapshot_request = SnapshotReadRequest::for_relational_record(
        RelationalBridgeRecordIdentityParts::entity(0, 4, 1),
        SnapshotReadContract::scalar(
            AspectKey::new("profile.name").unwrap(),
            ScalarAspectType::String,
        ),
    );
    let snapshot_identity = super::bridge_snapshot_identity_for_commit(CommitId(7), VersionId(7));
    catalog.register_snapshot(PublicationBridgeSnapshot::new(
        snapshot_identity.clone(),
        vec![SnapshotReadRecord::for_request(
            &snapshot_request,
            AspectValue::String("alice".into()),
        )],
    ));

    let envelope = catalog
        .load_committed_patch(
            RelationalCommittedPatchRequest::new(TruthCommitIdentity::from_relational_commit_id(7)),
            execution,
        )
        .expect("registered publication patch");
    let reader = catalog
        .open_snapshot(&snapshot_identity, execution)
        .expect("registered publication snapshot");

    assert_eq!(
        envelope.patch_identity().relational_patch_position(),
        Some(11)
    );
    assert_eq!(
        envelope.patch_body().canonical_items()[0]
            .aspect_key()
            .as_str(),
        "profile.name"
    );
    assert_eq!(reader.snapshot_identity(), snapshot_identity);
}

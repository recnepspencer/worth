use super::*;
use crate::history::data::{
    BranchId, CanonicalCommitAuthorityKind, CanonicalCommitEnvelopeDraft, CommitId,
    RelationalCommitReceipt, RelationalDescriptiveTouchGraph,
};
use crate::identity::data::{EntityId, PartitionId, VersionId};
use crate::publication::patch::data::*;
use crate::schema::data::*;
use crate::transactions::data::{MergedCommitPlan, RecordRef, TransactionId};

#[test]
fn completeness_recomputes_visibility_commitment() {
    let body = body(false);
    let (mut axes, descriptor, content) = binding(&body);
    assert!(original_is_complete(
        axes,
        &descriptor,
        content.accumulator,
        content,
        &body,
        true
    ));
    axes.visibility = RelationalBranchVisibilityCommitment::for_root(
        &self::body(true),
        axes.storage_root,
        axes.schema_root,
        axes.correctness_index,
    );
    assert!(
        !original_is_complete(axes, &descriptor, content.accumulator, content, &body, true),
        "completeness must reject a commitment to any different visible tuple"
    );
}

#[test]
fn completeness_rejects_each_reconstructed_binding_axis_mismatch() {
    let body = body(false);
    let (axes, descriptor, content) = binding(&body);
    for mutant in [
        RelationalBranchRootAxes {
            storage_version: 2,
            ..axes
        },
        RelationalBranchRootAxes {
            storage_root: [9; 32],
            ..axes
        },
        RelationalBranchRootAxes {
            schema_root: [9; 32],
            ..axes
        },
        RelationalBranchRootAxes {
            visibility: RelationalBranchVisibilityCommitment::from_checkpoint([9; 32]),
            ..axes
        },
    ] {
        assert!(!original_is_complete(
            mutant,
            &descriptor,
            content.accumulator,
            content,
            &body,
            true
        ));
    }
    for mutant in [
        RelationalBranchRootDescriptor::new([9; 32], axes.schema_root),
        RelationalBranchRootDescriptor::new(axes.storage_root, [9; 32]),
    ] {
        assert!(!original_is_complete(
            axes,
            &mutant,
            content.accumulator,
            content,
            &body,
            true
        ));
    }
    assert!(!original_is_complete(
        axes,
        &descriptor,
        [9; 32],
        content,
        &body,
        true
    ));
    assert!(!original_is_complete(
        axes,
        &descriptor,
        content.accumulator,
        content,
        &body,
        false
    ));
    let zero = ReconstructedContent {
        storage: CanonicalDigestId::new([0; 32]),
        ..content
    };
    let zero_axes = RelationalBranchRootAxes {
        storage_root: [0; 32],
        visibility: RelationalBranchVisibilityCommitment::for_root(
            &body,
            [0; 32],
            axes.schema_root,
            axes.correctness_index,
        ),
        ..axes
    };
    assert!(!original_is_complete(
        zero_axes,
        &RelationalBranchRootDescriptor::new([0; 32], axes.schema_root),
        zero.accumulator,
        zero,
        &body,
        true
    ));
}

fn binding(
    body: &CanonicalCommitEnvelope,
) -> (
    RelationalBranchRootAxes,
    RelationalBranchRootDescriptor,
    ReconstructedContent,
) {
    let schema = schema_authority_snapshot_digest_bytes(&body.schema_authority);
    let content = ReconstructedContent {
        storage: CanonicalDigestId::new([7; 32]),
        schema: CanonicalDigestId::new(schema),
        accumulator: [11; 32],
    };
    let correctness_index = super::super::RelationalRootCorrectnessIndex::AuthoritativeFallback;
    let axes = RelationalBranchRootAxes {
        storage_version: 1,
        storage_root: [7; 32],
        schema_root: schema,
        correctness_index,
        visibility: RelationalBranchVisibilityCommitment::for_root(
            body,
            [7; 32],
            schema,
            correctness_index,
        ),
    };
    (
        axes,
        RelationalBranchRootDescriptor::new([7; 32], schema),
        content,
    )
}

// Pure predicate inputs, using the same actual-original/body-control recipe
// as root_physical_linkage_tests. No altered value enters publication or recovery.
fn body(opaque: bool) -> CanonicalCommitEnvelope {
    use crate::diagnostics::data::*;
    use crate::lineage::data::*;
    let branch = BranchId("completeness-value".into());
    let draft = CanonicalCommitEnvelopeDraft::new(
        Default::default(),
        crate::history::data::CanonicalIndexDefinitionDelta::prepared(1, 1, vec![], None),
        RelationalCommitReceipt {
            commit_id: CommitId(1),
            version_id: VersionId(1),
            branch_id: branch.clone(),
            parents: vec![],
        },
        branch.clone(),
        CanonicalCommitAuthorityKind::VersionedTransaction,
        None,
        None,
        vec![],
        vec![],
        SchemaVersionId(1),
        RelationalSchemaRegistry::new().authority_snapshot(),
        MergedCommitPlan {
            transaction_id: TransactionId(1),
            merged_intents: vec![],
        },
        CanonicalAuthoritativePatch {
            ordering: PatchOrdering::CanonicalCommitOrder,
            publication_mode: PatchPublicationMode::CommitNative,
            authoritative_record_patches: vec![PublishedAuthoritativeRecordPatch {
                target: RecordRef::Entity(EntityId::new(PartitionId::main(), 1, 1)),
                structural_change: RecordStructuralChange::Updated,
                relation_birth: None,
                authoritative_patch: PublishedAuthoritativePatch::empty(),
                semantic_changes: vec![],
                contains_opaque_aspect: opaque,
                detail: PatchDetail::DenseBitset(vec![1]),
            }],
        },
        RelationalDescriptiveTouchGraph::exact(vec![]),
        RelationalDiagnosticArtifact::new(
            DiagnosticsScope::Replay,
            DiagnosticsArtifactKind::MinimalSummary,
            DeterminismExpectation::Required,
            vec![],
        ),
        LineageFinalizationArtifact::new(
            branch,
            FinalizedLineageEventBatch::new(vec![]),
            LineageDecisionLog::new(vec![]),
        )
        .publish(),
        crate::indexes::data::DerivedIndexArtifacts::default(),
        None,
        None,
        None,
        DescriptorSemanticsVersion(1),
    )
    .into_body();
    crate::tests::support::with_memory_record_publication(
        crate::tests::support::test_schema_registry(),
        "completeness-semantic-source",
        |world| {
            let root = world
                .runtime
                .history
                .branch_cell(&BranchId("main".into()))
                .unwrap()
                .root()
                .unwrap();
            let envelope = root.canonical_envelope().unwrap().clone();
            drop(root);
            drop(world);
            let mut envelope = std::sync::Arc::try_unwrap(envelope)
                .expect("the actual Memory source has retired its Original shares");
            *envelope.body_mut_for_test() = draft;
            envelope
        },
    )
}

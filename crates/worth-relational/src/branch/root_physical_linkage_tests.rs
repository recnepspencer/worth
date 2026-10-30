//! Immutable physical-linkage predicate inputs, never an admitted runtime world.
use super::*;
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticArtifact,
};
use crate::history::data::{
    BranchId, CanonicalCommitAuthorityKind, RelationalCommitReceipt,
    RelationalDescriptiveTouchGraph,
};
use crate::identity::data::VersionId;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::{
    FinalizedLineageEventBatch, LineageDecisionLog, LineageFinalizationArtifact,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PatchOrdering, PatchPublicationMode,
};
use crate::schema::data::{DescriptorSemanticsVersion, RelationalSchemaRegistry, SchemaVersionId};
use crate::transactions::data::{MergedCommitPlan, TransactionId};

#[test]
fn equal_envelope_bytes_cannot_replace_exact_physical_catalog_custody() {
    let envelope = Arc::new(envelope_input());
    let descriptor = RelationalBranchRootDescriptor::new([7; 32], [8; 32]);
    let axes = RelationalBranchRootAxes {
        storage_version: 1,
        storage_root: *descriptor.truth_root(),
        schema_root: *descriptor.schema_root(),
        correctness_index: RelationalRootCorrectnessIndex::AuthoritativeFallback,
        visibility: RelationalBranchVisibilityCommitment::for_root(
            &envelope,
            *descriptor.truth_root(),
            *descriptor.schema_root(),
            RelationalRootCorrectnessIndex::AuthoritativeFallback,
        ),
    };
    let mut root = (*RelationalBranchRoot::empty()).clone();
    root.committed = Some(RelationalCommittedBranchRoot {
        descriptor,
        canonical_envelope: Arc::clone(&envelope),
        axes,
        publication_cost: Default::default(),
    });
    let root = Arc::new(root);
    let artifact = crate::history::RelationalCommitArtifact::from_envelope_with_root(
        Arc::clone(&envelope),
        Arc::clone(&root),
    )
    .unwrap();
    let mut distinct_custody = (*root).clone();
    distinct_custody
        .committed
        .as_mut()
        .unwrap()
        .canonical_envelope = Arc::new((*envelope).clone());
    let distinct_custody = Arc::new(distinct_custody);
    assert!(artifact.links_root(&root));
    assert!(!artifact.links_root(&distinct_custody));
    assert!(!distinct_custody.links_envelope(&envelope));
    // The differing pointer is the only changed predicate input. No runtime,
    // publication, checkpoint or operational admission is claimed here.
    assert_eq!(root.descriptor(), distinct_custody.descriptor());
}

fn envelope_input() -> CanonicalCommitEnvelope {
    CanonicalCommitEnvelope::new(
        RelationalCommitReceipt {
            commit_id: CommitId(1),
            version_id: VersionId(1),
            branch_id: BranchId("predicate".into()),
            parents: vec![],
        },
        BranchId("predicate".into()),
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
            authoritative_record_patches: vec![],
        },
        RelationalDescriptiveTouchGraph::exact(vec![]),
        RelationalDiagnosticArtifact::new(
            DiagnosticsScope::Replay,
            DiagnosticsArtifactKind::MinimalSummary,
            DeterminismExpectation::Required,
            vec![],
        ),
        LineageFinalizationArtifact::new(
            BranchId("predicate".into()),
            FinalizedLineageEventBatch::new(vec![]),
            LineageDecisionLog::new(vec![]),
        )
        .publish(),
        DerivedIndexArtifacts::default(),
        None,
        None,
        None,
        DescriptorSemanticsVersion(1),
    )
}

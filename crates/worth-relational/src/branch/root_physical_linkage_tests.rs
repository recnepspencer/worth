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
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let envelope = Arc::new(envelope_input(execution));
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
        commit_change: None,
        descriptor,
        evidence: RelationalRootCommitEvidence::original_for_request(
            execution,
            Arc::clone(&envelope),
        )
        .unwrap(),
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
    distinct_custody.committed.as_mut().unwrap().evidence =
        RelationalRootCommitEvidence::original_for_request(
            execution,
            Arc::new(envelope.copy_original_for_test(execution)),
        )
        .unwrap();
    let distinct_custody = Arc::new(distinct_custody);
    assert!(artifact.links_root(&root));
    assert!(!artifact.links_root(&distinct_custody));
    assert!(!distinct_custody.links_envelope(&envelope));
    // The differing pointer is the only changed predicate input. No runtime,
    // publication, checkpoint or operational admission is claimed here.
    assert_eq!(root.descriptor(), distinct_custody.descriptor());
}

fn envelope_input(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CanonicalCommitEnvelope {
    let draft = crate::history::data::CanonicalCommitEnvelopeDraft::new(
        Default::default(),
        crate::history::data::CanonicalIndexDefinitionDelta::prepared(1, 1, Vec::new(), None),
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
    );
    // Pure predicate/inspection input: keep an actual original semantic image,
    // then edit only this test body. No publication or readmission is claimed.
    let runtime = crate::tests::support::runtime_with_test_schema();
    let produced = crate::tests::support::create_entity_outcome(
        execution,
        &runtime,
        "predicate-semantic-source",
    );
    let envelope = produced.publication().envelope.clone().into_arc();
    crate::tests::support::release_test_commit_snapshot(&runtime, &produced);
    drop(produced);
    drop(runtime);
    let mut envelope = std::sync::Arc::try_unwrap(envelope)
        .expect("source owner and result released their actual envelope shares");
    *envelope.body_mut_for_test() = draft.into_body();
    envelope
}

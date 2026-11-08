//! Pure codec/predicate inputs, not an admitted transaction or runtime world.
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticArtifact,
};
use crate::history::data::{
    BranchId, CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, CommitId,
    RelationalCommitReceipt, RelationalDescriptiveTouchGraph,
};
use crate::identity::data::VersionId;
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::{
    FinalizedLineageEventBatch, LineageDecisionLog, LineageFinalizationArtifact,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PatchOrdering, PatchPublicationMode,
};
use crate::schema::data::{
    LoweredSchemaTransitionPlan, RelationalSchemaRegistry, SchemaTransitionArtifact,
};
use crate::transactions::data::{MergedCommitPlan, TransactionId};

pub(crate) fn canonical_schema_envelope_input(
    registry: &RelationalSchemaRegistry,
    transition: Option<LoweredSchemaTransitionPlan>,
) -> CanonicalCommitEnvelope {
    let continuation = transition
        .as_ref()
        .map(|t| t.continuation_descriptor.clone());
    let reconciliation = transition
        .as_ref()
        .map(|t| t.reconciliation_descriptor.clone());
    let artifact = transition.map(|t| {
        let p = t.validated.proposed;
        SchemaTransitionArtifact::new(
            p.source_schema_id,
            p.source_schema_version_id,
            p.target_schema_id,
            p.target_schema_version_id,
            p.diff_atoms,
            t.continuation_descriptor,
            t.reconciliation_descriptor,
        )
    });
    CanonicalCommitEnvelope::new(
        Default::default(),
        RelationalCommitReceipt {
            commit_id: CommitId(2),
            version_id: VersionId(2),
            branch_id: BranchId("schema-input".into()),
            parents: vec![CommitId(1)],
        },
        BranchId("schema-input".into()),
        CanonicalCommitAuthorityKind::VersionedTransaction,
        None,
        None,
        vec![],
        vec![],
        registry.primary_schema_version_id(),
        registry.authority_snapshot(),
        MergedCommitPlan {
            transaction_id: TransactionId(2),
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
            BranchId("schema-input".into()),
            FinalizedLineageEventBatch::new(vec![]),
            LineageDecisionLog::new(vec![]),
        )
        .publish(),
        DerivedIndexArtifacts::default(),
        artifact,
        continuation,
        reconciliation,
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
    )
}

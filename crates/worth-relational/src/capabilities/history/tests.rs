use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::{CommitEnvelopeSource, HistorySource, PatchStreamCommitRef, PatchStreamSource};
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticArtifact,
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::history::data::{
    CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, RelationalDescriptiveTouchGraph,
};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::lineage::data::{
    FinalizedLineageEventBatch, LineageDecisionLog, LineageFinalizationArtifact,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PatchDetail, PatchOrdering, PatchPublicationMode,
    PatchStreamPosition, PublishedAuthoritativeRecordPatch, RecordStructuralChange,
};
use crate::schema::data::{DescriptorSemanticsVersion, RelationalSchemaRegistry, SchemaVersionId};
use crate::transactions::data::{MergedCommitPlan, RecordRef, TransactionId};

struct FakeHistorySource {
    envelopes: BTreeMap<CommitId, Arc<CanonicalCommitEnvelope>>,
    patch_stream_index: BTreeMap<PatchStreamPosition, CommitId>,
}

impl CommitEnvelopeSource for FakeHistorySource {
    fn commit_envelope(&self, commit_id: CommitId) -> Option<CanonicalCommitEnvelope> {
        self.envelopes
            .get(&commit_id)
            .map(|envelope| envelope.as_ref().clone())
    }
}

impl PatchStreamSource for FakeHistorySource {
    fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition> {
        self.patch_stream_index
            .last_key_value()
            .map(|(position, _)| *position)
    }

    fn commit_id_at_patch_stream_position(
        &self,
        position: PatchStreamPosition,
    ) -> Option<CommitId> {
        self.patch_stream_index.get(&position).copied()
    }

    fn patch_stream_commits_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<PatchStreamCommitRef> {
        let start = after_position
            .map(std::ops::Bound::Excluded)
            .unwrap_or(std::ops::Bound::Unbounded);
        self.patch_stream_index
            .range((start, std::ops::Bound::Unbounded))
            .map(|(position, commit_id)| PatchStreamCommitRef {
                position: *position,
                commit_id: *commit_id,
            })
            .take(max_commits)
            .collect()
    }
}

impl HistorySource for FakeHistorySource {
    fn branch_head_ref(&self, _branch_id: &BranchId) -> Option<RelationalCommitReceipt> {
        None
    }

    fn authoritative_commit_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.envelopes.values().map(Arc::clone).collect()
    }
}

fn commit_envelope(commit_id: u64, version_id: u64) -> CanonicalCommitEnvelope {
    CanonicalCommitEnvelope::new(
        RelationalCommitReceipt {
            commit_id: CommitId(commit_id),
            version_id: crate::identity::data::VersionId(version_id),
            branch_id: BranchId("main".to_string()),
            parents: vec![],
        },
        BranchId("main".to_string()),
        CanonicalCommitAuthorityKind::VersionedTransaction,
        None,
        None,
        vec![],
        vec![],
        SchemaVersionId(1),
        RelationalSchemaRegistry::new().authority_snapshot(),
        MergedCommitPlan {
            transaction_id: TransactionId(commit_id),
            merged_intents: vec![],
        },
        CanonicalAuthoritativePatch {
            ordering: PatchOrdering::CanonicalCommitOrder,
            publication_mode: PatchPublicationMode::CommitNative,
            authoritative_record_patches: vec![PublishedAuthoritativeRecordPatch {
                target: RecordRef::Entity(crate::identity::data::EntityId::new(
                    crate::identity::data::PartitionId::main(),
                    commit_id,
                    0,
                )),
                structural_change: RecordStructuralChange::Updated,
                authoritative_patch:
                    crate::publication::patch::data::PublishedAuthoritativePatch::empty(),
                semantic_changes: Vec::new(),
                contains_opaque_aspect: false,
                detail: PatchDetail::DenseBitset(vec![]),
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
            BranchId("main".to_string()),
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

#[test]
fn committed_version_outside_closure_ignores_closure_members_and_detects_external_predecessor() {
    let history = FakeHistorySource {
        envelopes: BTreeMap::from([
            (CommitId(1), Arc::new(commit_envelope(1, 1))),
            (CommitId(2), Arc::new(commit_envelope(2, 2))),
        ]),
        patch_stream_index: BTreeMap::from([
            (PatchStreamPosition(1), CommitId(1)),
            (PatchStreamPosition(2), CommitId(2)),
        ]),
    };

    assert!(history.has_committed_version_at_or_before_outside_closure(
        crate::identity::data::VersionId(2),
        &BTreeSet::from([CommitId(2)]),
    ));
    assert!(!history.has_committed_version_at_or_before_outside_closure(
        crate::identity::data::VersionId(2),
        &BTreeSet::from([CommitId(1), CommitId(2)]),
    ));
}

#[test]
fn patch_stream_source_resolves_commit_envelopes_from_positions() {
    let history = FakeHistorySource {
        envelopes: BTreeMap::from([
            (CommitId(1), Arc::new(commit_envelope(1, 1))),
            (CommitId(2), Arc::new(commit_envelope(2, 2))),
        ]),
        patch_stream_index: BTreeMap::from([
            (PatchStreamPosition(1), CommitId(1)),
            (PatchStreamPosition(2), CommitId(2)),
        ]),
    };

    let envelope = history
        .commit_envelope_at_patch_stream_position(PatchStreamPosition(2))
        .expect("patch stream envelope");

    assert_eq!(
        history.latest_patch_stream_position(),
        Some(PatchStreamPosition(2))
    );
    assert_eq!(envelope.commit.commit_id, CommitId(2));
    assert!(history.contains_patch_stream_position(PatchStreamPosition(1)));
}

#[test]
fn patch_stream_source_exposes_exact_position_windows() {
    let history = FakeHistorySource {
        envelopes: BTreeMap::from([
            (CommitId(1), Arc::new(commit_envelope(1, 1))),
            (CommitId(2), Arc::new(commit_envelope(2, 2))),
        ]),
        patch_stream_index: BTreeMap::from([
            (PatchStreamPosition(1), CommitId(1)),
            (PatchStreamPosition(2), CommitId(2)),
        ]),
    };

    assert_eq!(
        history.patch_stream_commits_after(Some(PatchStreamPosition(1)), 8),
        vec![PatchStreamCommitRef {
            position: PatchStreamPosition(2),
            commit_id: CommitId(2),
        }],
    );
}

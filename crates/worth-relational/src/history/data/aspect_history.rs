mod diagnostic_fields;

use serde::{Deserialize, Serialize};

use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::identity::data::{EntityId, LineageId, RelationId, VersionId};
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;
use crate::visibility::materialization::read_records::ProjectionAspectFilter;
use worth_foundational::facade::AspectKey;

use super::{BranchId, CommitId};
use diagnostic_fields::aspect_history_resolution_trace_fields;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectHistoryOrigin {
    pub commit_id: CommitId,
    pub version_id: VersionId,
    pub branch_id: BranchId,
    pub target: RecordRef,
    pub structural_change: RecordStructuralChange,
    pub changed_aspects: Vec<AspectKey>,
    pub contains_opaque_aspect: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AspectResolutionContext {
    DirectRecordHistory,
    ResolvedViaLineage {
        start_lineage_id: LineageId,
        traversed_event_ids: Vec<u64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectHistoryEntry {
    pub origin: AspectHistoryOrigin,
    pub resolution: AspectResolutionContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageAspectHistory {
    pub requested_branch: BranchId,
    pub start_lineage_id: LineageId,
    pub resolved_lineage_chain: Vec<LineageId>,
    pub entries: Vec<AspectHistoryEntry>,
    pub traversed_event_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HistoryAspectQueryTarget {
    Entity(EntityId),
    Relation(RelationId),
    Lineage(LineageId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectHistoryResolutionTrace {
    pub requested_target: HistoryAspectQueryTarget,
    pub branch_id: BranchId,
    pub filter: Option<ProjectionAspectFilter>,
    pub resolved_aspects: Vec<AspectKey>,
    pub searched_commit_span: Option<AspectHistoryCommitSpan>,
    pub searched_lineage_event_span: Option<AspectHistoryLineageEventSpan>,
    pub returned_entries: u64,
    pub traversed_commits: u64,
    pub traversed_lineage_events: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectHistoryCommitSpan {
    pub first_commit_id: CommitId,
    pub last_commit_id: CommitId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectHistoryLineageEventSpan {
    pub first_event_id: u64,
    pub last_event_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct AspectHistoryQueryResult {
    pub entries: Vec<AspectHistoryEntry>,
    pub trace: AspectHistoryResolutionTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct LineageAspectHistoryQueryResult {
    pub history: Option<LineageAspectHistory>,
    pub trace: AspectHistoryResolutionTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct AspectHistoryDigest {
    pub requested_target: HistoryAspectQueryTarget,
    pub branch_id: BranchId,
    pub resolved_aspects: Vec<AspectKey>,
    pub entry_count: u64,
    pub opaque_aspect_entry_count: u64,
    pub traversed_commits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub struct LineageAspectResolutionDigest {
    pub requested_target: HistoryAspectQueryTarget,
    pub branch_id: BranchId,
    pub resolved_aspects: Vec<AspectKey>,
    pub entry_count: u64,
    pub opaque_aspect_entry_count: u64,
    pub traversed_commits: u64,
    pub traversed_lineage_events: u64,
    pub resolved_lineage_chain_len: u64,
}

impl AspectHistoryResolutionTrace {
    pub fn diagnostics_scope(&self) -> DiagnosticsScope {
        match self.requested_target {
            HistoryAspectQueryTarget::Lineage(_) => DiagnosticsScope::Lineage,
            HistoryAspectQueryTarget::Entity(_) | HistoryAspectQueryTarget::Relation(_) => {
                DiagnosticsScope::History
            }
        }
    }

    pub fn diagnostic_artifact(&self) -> RelationalDiagnosticArtifact {
        let fields = aspect_history_resolution_trace_fields(self);
        RelationalDiagnosticArtifact::new(
            self.diagnostics_scope(),
            DiagnosticsArtifactKind::DetailedTrace,
            DeterminismExpectation::Required,
            vec![RelationalDiagnosticsEntry::new(
                match self.requested_target {
                    HistoryAspectQueryTarget::Lineage(_) => {
                        DiagnosticCode::LineageAspectHistoryResolved
                    }
                    HistoryAspectQueryTarget::Entity(_) | HistoryAspectQueryTarget::Relation(_) => {
                        DiagnosticCode::AspectHistoryResolved
                    }
                },
                "aspect history resolved from committed aspect truth",
                fields,
            )],
        )
    }
}

impl AspectHistoryQueryResult {
    pub fn aspect_history_digest(&self) -> AspectHistoryDigest {
        AspectHistoryDigest {
            requested_target: self.trace.requested_target.clone(),
            branch_id: self.trace.branch_id.clone(),
            resolved_aspects: self.trace.resolved_aspects.clone(),
            entry_count: self.entries.len() as u64,
            opaque_aspect_entry_count: self
                .entries
                .iter()
                .filter(|entry| entry.origin.contains_opaque_aspect)
                .count() as u64,
            traversed_commits: self.trace.traversed_commits,
        }
    }
}

impl LineageAspectHistoryQueryResult {
    pub fn lineage_aspect_resolution_digest(&self) -> LineageAspectResolutionDigest {
        LineageAspectResolutionDigest {
            requested_target: self.trace.requested_target.clone(),
            branch_id: self.trace.branch_id.clone(),
            resolved_aspects: self.trace.resolved_aspects.clone(),
            entry_count: self
                .history
                .as_ref()
                .map(|history| history.entries.len() as u64)
                .unwrap_or(0),
            opaque_aspect_entry_count: self
                .history
                .as_ref()
                .map(|history| {
                    history
                        .entries
                        .iter()
                        .filter(|entry| entry.origin.contains_opaque_aspect)
                        .count() as u64
                })
                .unwrap_or(0),
            traversed_commits: self.trace.traversed_commits,
            traversed_lineage_events: self.trace.traversed_lineage_events,
            resolved_lineage_chain_len: self
                .history
                .as_ref()
                .map(|history| history.resolved_lineage_chain.len() as u64)
                .unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::history::data::{AspectHistoryDigest, BranchId, HistoryAspectQueryTarget};
    use crate::identity::data::{EntityId, PartitionId};
    use crate::publication::patch::data::ordered_aspect_keys;

    #[test]
    fn aspect_history_digest_preserves_wide_counts_as_u64() {
        let digest = AspectHistoryDigest {
            requested_target: HistoryAspectQueryTarget::Entity(EntityId::new(PartitionId(1), 0, 1)),
            branch_id: BranchId("main".to_string()),
            resolved_aspects: ordered_aspect_keys([]),
            entry_count: u64::from(u32::MAX) + 17,
            opaque_aspect_entry_count: 9,
            traversed_commits: u64::from(u32::MAX) + 23,
        };

        assert_eq!(digest.entry_count, u64::from(u32::MAX) + 17);
        assert_eq!(digest.opaque_aspect_entry_count, 9);
        assert_eq!(digest.traversed_commits, u64::from(u32::MAX) + 23);
    }
}

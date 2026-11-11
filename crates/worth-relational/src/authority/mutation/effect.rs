use crate::diagnostics::data::RelationalDiagnosticsEntry;
use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::RecordRef;

use super::canonical_deltas::{CanonicalRecordAspectDelta, FoundationalPatchFragment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdjacencyDeltaKind {
    Created { source: EntityId, target: EntityId },
    Deleted { source: EntityId, target: EntityId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdjacencyDelta {
    pub(crate) relation_id: RelationId,
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) kind: AdjacencyDeltaKind,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct MutationPublicationEffect {
    pub(crate) has_entity_deletions: bool,
    pub(crate) changed_records: Vec<RecordRef>,
    pub(crate) canonical_deltas: Vec<CanonicalRecordAspectDelta>,
    pub(crate) patch_fragments: Vec<FoundationalPatchFragment>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct MutationDiagnosticsEffect {
    pub(crate) entries: Vec<RelationalDiagnosticsEntry>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct MutationAdjacencyEffect {
    pub(crate) deltas: Vec<AdjacencyDelta>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct MutationEffect {
    pub(crate) publication: MutationPublicationEffect,
    pub(crate) diagnostics: MutationDiagnosticsEffect,
    pub(crate) adjacency: MutationAdjacencyEffect,
}

impl MutationEffect {
    pub(crate) fn with_capacity(change_count: usize, diagnostic_count: usize) -> Self {
        Self {
            publication: MutationPublicationEffect {
                has_entity_deletions: false,
                changed_records: Vec::with_capacity(change_count),
                canonical_deltas: Vec::with_capacity(change_count),
                patch_fragments: Vec::with_capacity(change_count),
            },
            diagnostics: MutationDiagnosticsEffect {
                entries: Vec::with_capacity(diagnostic_count),
            },
            adjacency: MutationAdjacencyEffect {
                deltas: Vec::with_capacity(change_count),
            },
        }
    }

    pub(crate) fn accumulate(&mut self, child: MutationEffect) {
        self.publication.has_entity_deletions |= child.publication.has_entity_deletions;
        self.publication
            .changed_records
            .extend(child.publication.changed_records);
        self.publication
            .canonical_deltas
            .extend(child.publication.canonical_deltas);
        self.publication
            .patch_fragments
            .extend(child.publication.patch_fragments);
        self.diagnostics.entries.extend(child.diagnostics.entries);
        self.adjacency.deltas.extend(child.adjacency.deltas);
    }
}

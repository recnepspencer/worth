use crate::contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::transactions::data::primitives::RecordRef;

use super::canonical_deltas::{CanonicalRecordAspectDelta, FoundationalPatchFragment};

pub use crate::contracts::authority::mutation::adjacency_delta::{
    AdjacencyDelta, AdjacencyDeltaKind,
};

#[derive(Debug, Clone, Default)]
pub struct MutationPublicationEffect {
    pub has_entity_deletions: bool,
    pub changed_records: Vec<RecordRef>,
    pub canonical_deltas: Vec<CanonicalRecordAspectDelta>,
    pub patch_fragments: Vec<FoundationalPatchFragment>,
    pub entity_replacements: Vec<(EntityId, EntityId)>,
}

#[derive(Debug, Clone, Default)]
pub struct MutationDiagnosticsEffect {
    pub entries: Vec<RelationalDiagnosticsEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct MutationAdjacencyEffect {
    pub deltas: Vec<AdjacencyDelta>,
}

#[derive(Debug, Clone, Default)]
pub struct MutationEffect {
    pub publication: MutationPublicationEffect,
    pub diagnostics: MutationDiagnosticsEffect,
    pub adjacency: MutationAdjacencyEffect,
}

impl MutationEffect {
    pub(crate) fn with_capacity(change_count: usize, diagnostic_count: usize) -> Self {
        Self {
            publication: MutationPublicationEffect {
                has_entity_deletions: false,
                changed_records: Vec::with_capacity(change_count),
                canonical_deltas: Vec::with_capacity(change_count),
                patch_fragments: Vec::with_capacity(change_count),
                entity_replacements: Vec::new(),
            },
            diagnostics: MutationDiagnosticsEffect {
                entries: Vec::with_capacity(diagnostic_count),
            },
            adjacency: MutationAdjacencyEffect {
                deltas: Vec::with_capacity(change_count),
            },
        }
    }

    pub fn accumulate(&mut self, child: MutationEffect) {
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
        self.publication
            .entity_replacements
            .extend(child.publication.entity_replacements);
        self.diagnostics.entries.extend(child.diagnostics.entries);
        self.adjacency.deltas.extend(child.adjacency.deltas);
    }
}

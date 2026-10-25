use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

#[cfg(test)]
mod tests;

/// Opaque native bytes for one complete Relational recovery checkpoint.
///
/// Construction accepts untrusted bytes deliberately: admission belongs to
/// the recovery authority, where codec integrity and runtime continuity are
/// checked together. Callers can retain or transport the bytes, but cannot
/// inspect or selectively rewrite Relational truth through this type.
/// Clones share immutable byte backing and retain the same selected region;
/// they do not clone or confer recovery authority.
pub struct RelationalNativeCheckpoint {
    bytes: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
    captured_sections: Option<NativeCheckpointSectionBytes>,
    captured_coverage: Option<NativeCheckpointCoverage>,
    members: Arc<[super::RelationalCheckpointMember]>,
    publications: Arc<[crate::storage::residency::RelationalCheckpointRegionPublication]>,
}

/// What a captured checkpoint covers: its last commit position and its last
/// branch transition sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeCheckpointCoverage {
    position: crate::publication::patch::data::PatchStreamPosition,
    transitions: crate::branch::RelationalBranchTransitionSequence,
}

/// Actual MessagePack value bytes emitted during one native capture. The
/// remainder contains map keys, framing, and small checkpoint metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeCheckpointSectionBytes {
    pub total: usize,
    pub envelopes: usize,
    pub branch_roots: usize,
    pub branch_cells: usize,
    pub partition_mirror: usize,
    pub derived_indexes: usize,
    pub framing_and_metadata: usize,
}

impl Clone for RelationalNativeCheckpoint {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            region: self.region.clone(),
            captured_sections: self.captured_sections,
            captured_coverage: self.captured_coverage,
            members: self.members.clone(),
            publications: self.publications.clone(),
        }
    }
}

impl std::fmt::Debug for RelationalNativeCheckpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalNativeCheckpoint")
            .field("byte_len", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for RelationalNativeCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for RelationalNativeCheckpoint {}

impl RelationalNativeCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        let bytes = bytes.into();
        let region = 0..bytes.len();
        Self {
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes)),
            region,
            captured_sections: None,
            captured_coverage: None,
            members: Arc::from([]),
            publications: Arc::from([]),
        }
    }

    /// Retain an embedded native payload without copying its enclosing buffer.
    /// The caller shares immutable backing; the selected range stays private.
    /// Recovery still authenticates and readmits the selected native bytes.
    pub fn from_untrusted_bytes_region(
        bytes: ExecutionImmutableBytes,
        region: std::ops::Range<usize>,
    ) -> Result<Self, &'static str> {
        if bytes.get(region.clone()).is_none() {
            return Err("native checkpoint byte region is out of bounds");
        }
        Ok(Self {
            bytes,
            region,
            captured_sections: None,
            captured_coverage: None,
            members: Arc::from([]),
            publications: Arc::from([]),
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }

    /// Members to publish atomically with the manifest. Inline artifacts also
    /// expose their selected members; their bytes already contain the same set.
    pub fn members(&self) -> &[super::RelationalCheckpointMember] {
        &self.members
    }

    pub(crate) fn with_captured_members(
        mut self,
        members: Vec<super::RelationalCheckpointMember>,
    ) -> Self {
        self.members = members.into();
        self
    }

    /// Confirm that the manifest and its complete member set were selected in
    /// one durable publication. The home calls this only after that decision,
    /// even if its subsequent physical checkpoint is interrupted.
    pub fn acknowledge_publication(&self) {
        for publication in self.publications.iter() {
            publication.acknowledge();
        }
    }

    pub(crate) fn with_region_publications(
        mut self,
        publications: Vec<crate::storage::residency::RelationalCheckpointRegionPublication>,
    ) -> Self {
        self.publications = publications.into();
        self
    }

    /// Available only on a checkpoint captured by this runtime, not bytes
    /// received from a caller or an enclosing checkpoint container.
    pub fn captured_sections(&self) -> Option<NativeCheckpointSectionBytes> {
        self.captured_sections
    }

    pub(crate) fn from_captured_bytes(
        bytes: ExecutionImmutableBytes,
        sections: NativeCheckpointSectionBytes,
        position: crate::publication::patch::data::PatchStreamPosition,
        transitions: crate::branch::RelationalBranchTransitionSequence,
    ) -> Self {
        let region = 0..bytes.len();
        Self {
            bytes,
            region,
            members: Arc::from([]),
            publications: Arc::from([]),
            captured_sections: Some(sections),
            captured_coverage: Some(NativeCheckpointCoverage {
                position,
                transitions,
            }),
        }
    }

    /// The last canonical position this checkpoint includes, known only for a
    /// checkpoint this process captured. A medium files the checkpoint under
    /// this position; recovery checks the decoded checkpoint against it.
    pub fn covered_position(&self) -> Option<crate::publication::patch::data::PatchStreamPosition> {
        self.captured_coverage.map(|coverage| coverage.position)
    }

    /// The last branch transition sequence this checkpoint includes, known
    /// only for a checkpoint this process captured. A medium files it beside
    /// the covered position; recovery checks the decoded checkpoint against it.
    pub fn covered_branch_transitions(
        &self,
    ) -> Option<crate::branch::RelationalBranchTransitionSequence> {
        self.captured_coverage.map(|coverage| coverage.transitions)
    }
}

use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

mod component_custody;
mod publication_binding;
mod region_publications;
use region_publications::CapturedRegionPublications;
#[cfg(test)]
mod tests;
use component_custody::ComponentCaptureCustody;
pub(in crate::durability) use component_custody::OriginalCheckpointRead;

/// Opaque native bytes for one complete Relational recovery checkpoint.
///
/// Construction accepts untrusted bytes deliberately: admission belongs to
/// the recovery authority, where codec integrity and runtime continuity are
/// checked together. Callers can retain or transport the bytes, but cannot
/// inspect or selectively rewrite Relational truth through this type.
/// Clones share immutable byte backing and retain the same selected region;
/// they do not clone or confer recovery authority.
pub struct RelationalNativeCheckpoint {
    component_custody: ComponentCaptureCustody,
    bytes: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
    captured_sections: Option<NativeCheckpointSectionBytes>,
    captured_coverage: Option<NativeCheckpointCoverage>,
    members: super::CapturedCheckpointMembers,
    publications: CapturedRegionPublications,
    manifest_publication: Option<crate::runtime::RelationalCheckpointManifestPublication>,
    canonical_publication:
        Option<crate::durability::checkpoint_history::publication::CanonicalCheckpointPublication>,
    publication: Option<Arc<crate::durability::medium::CheckpointPublication>>,
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
            component_custody: self.component_custody.clone(),
            bytes: self.bytes.clone(),
            region: self.region.clone(),
            captured_sections: self.captured_sections,
            captured_coverage: self.captured_coverage,
            members: self.members.clone(),
            publications: self.publications.clone(),
            manifest_publication: self.manifest_publication.clone(),
            canonical_publication: self.canonical_publication.clone(),
            publication: self.publication.clone(),
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
            component_custody: ComponentCaptureCustody::default(),
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes)),
            region,
            captured_sections: None,
            captured_coverage: None,
            members: Default::default(),
            publications: Default::default(),
            manifest_publication: None,
            canonical_publication: None,
            publication: None,
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
            component_custody: ComponentCaptureCustody::default(),
            bytes,
            region,
            captured_sections: None,
            captured_coverage: None,
            members: Default::default(),
            publications: Default::default(),
            manifest_publication: None,
            canonical_publication: None,
            publication: None,
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }

    /// Members to publish atomically with the manifest. Inline artifacts also
    /// expose their selected members; their bytes already contain the same set.
    pub fn members(&self) -> &[super::RelationalCheckpointMember] {
        self.members.entries()
    }

    pub(crate) fn with_component_holds(
        mut self,
        holds: Vec<crate::branch::AdmittedRelationalBranchBasis>,
    ) -> Self {
        self.component_custody = ComponentCaptureCustody::new(holds);
        self
    }

    pub(in crate::durability) fn with_complete_component_holds(
        mut self,
        holds: crate::durability::authority::checkpoint_components::CompleteComponentHolds,
        region_cells: worth_execution::ExecutionArray<
            Arc<crate::storage::residency::RelationalRegionResidency>,
        >,
        original: Option<OriginalCheckpointRead>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::RelationalNativeCheckpointCaptureDenial> {
        self.component_custody =
            ComponentCaptureCustody::prepare_for_request(holds, region_cells, original, execution)
                .map_err(super::RelationalNativeCheckpointCaptureDenial::Request)?;
        Ok(self)
    }

    pub(crate) fn retains_reconstructed_regions(&self) -> Result<bool, ()> {
        self.component_custody.retains_reconstructed_regions()
    }

    pub(crate) fn release_component_holds(&self) {
        self.component_custody.release();
    }

    pub(crate) fn with_captured_members(
        mut self,
        members: super::CapturedCheckpointMembers,
    ) -> Self {
        self.members = members;
        self
    }

    /// Ask the attached medium about this original capture, then acknowledge
    /// its actual selection. Progress remains owned across caller interruption.
    pub fn acknowledge_selected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), crate::durability::medium::RelationalCheckpointAcknowledgementDenial> {
        self.publication.as_ref().ok_or(crate::durability::medium::RelationalCheckpointAcknowledgementDenial::CaptureNotBound)?
            .advance(execution,context)
    }

    pub fn publication_subject(
        &self,
    ) -> Option<&crate::durability::medium::OpaqueCapturedCheckpointSubject> {
        self.publication
            .as_ref()
            .map(|publication| publication.subject())
    }

    /// Admission of an adapter's actual read source precedes physical effects.
    /// This does not certify selection or mint acknowledgement authority.
    pub fn validate_publication_source(
        &self,
        source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<(), crate::durability::medium::RelationalCheckpointAcknowledgementDenial> {
        use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Denial;
        if self.publication.is_none() {
            return Err(Denial::CaptureNotBound);
        }
        if !self
            .manifest_publication
            .as_ref()
            .is_some_and(|publication| publication.source_matches(source))
        {
            return Err(Denial::ForeignSource);
        }
        self.manifest_publication
            .as_ref()
            .expect("validated bound capture")
            .validate_coverage()?;
        Ok(())
    }

    pub(crate) fn canonical_publication(
        &self,
    ) -> Option<&crate::durability::checkpoint_history::publication::CanonicalCheckpointPublication>
    {
        self.canonical_publication.as_ref()
    }
    pub(crate) fn with_canonical_publication(
        mut self,
        publication: Option<
            crate::durability::checkpoint_history::publication::CanonicalCheckpointPublication,
        >,
    ) -> Self {
        self.canonical_publication = publication;
        self
    }

    pub(crate) fn manifest_publication(
        &self,
    ) -> Option<&crate::runtime::RelationalCheckpointManifestPublication> {
        self.manifest_publication.as_ref()
    }
    pub(crate) fn region_publications(
        &self,
    ) -> &[crate::storage::residency::RelationalCheckpointRegionPublication] {
        self.publications.entries()
    }

    pub(crate) fn with_region_publications(
        mut self,
        publications: Vec<crate::storage::residency::RelationalCheckpointRegionPublication>,
    ) -> Self {
        self.publications = CapturedRegionPublications::from_ordinary(publications);
        self
    }

    pub(in crate::durability) fn with_complete_region_publications(
        mut self,
        count: usize,
        publications: impl IntoIterator<
            Item = crate::storage::residency::RelationalCheckpointRegionPublication,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::RelationalNativeCheckpointCaptureDenial> {
        self.publications =
            CapturedRegionPublications::capture_for_request(count, publications, execution)?;
        Ok(self)
    }

    pub(crate) fn with_manifest_publication(
        mut self,
        publication: Option<crate::runtime::RelationalCheckpointManifestPublication>,
    ) -> Self {
        self.manifest_publication = publication;
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
            component_custody: ComponentCaptureCustody::default(),
            bytes,
            region,
            members: Default::default(),
            publications: Default::default(),
            manifest_publication: None,
            canonical_publication: None,
            publication: None,
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

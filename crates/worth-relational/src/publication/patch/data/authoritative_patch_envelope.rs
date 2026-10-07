use crate::publication::patch::data::{
    PatchDetail, PatchOrdering, PatchPublicationMode, PatchStreamPosition,
    PublishedAuthoritativePatch, RecordStructuralChange,
};
use crate::transactions::data::RecordRef;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedAuthoritativeRecordPatch {
    pub target: RecordRef,
    pub structural_change: RecordStructuralChange,
    pub authoritative_patch: PublishedAuthoritativePatch,
    #[serde(default)]
    pub semantic_changes: Vec<super::PublishedAuthoritativeAspectChange>,
    pub contains_opaque_aspect: bool,
    pub detail: PatchDetail,
}

impl PublishedAuthoritativeRecordPatch {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.semantic_changes.capacity() as u64)
            .saturating_mul(std::mem::size_of::<super::PublishedAuthoritativeAspectChange>() as u64)
            .saturating_add(
                self.semantic_changes
                    .iter()
                    .map(super::PublishedAuthoritativeAspectChange::owned_allocation_capacity_bytes)
                    .sum::<u64>(),
            )
            .saturating_add(self.authoritative_patch.owned_allocation_capacity_bytes())
            .saturating_add(self.detail.owned_allocation_capacity_bytes())
    }

    pub fn authoritative_changed_aspect_keys(&self) -> impl Iterator<Item = &AspectKey> {
        self.authoritative_patch.changed_aspect_keys()
    }

    pub fn authoritative_changed_aspects(&self) -> Vec<AspectKey> {
        self.authoritative_patch.changed_aspects()
    }

    pub fn canonicalized(&self) -> Self {
        Self {
            target: self.target.clone(),
            structural_change: self.structural_change,
            authoritative_patch: self.authoritative_patch.canonicalized(),
            semantic_changes: {
                let mut changes = self.semantic_changes.clone();
                changes.sort_by_key(super::PublishedAuthoritativeAspectChange::canonical_key);
                changes.dedup();
                changes
            },
            contains_opaque_aspect: self.contains_opaque_aspect,
            detail: self.detail.canonicalized(),
        }
    }

    pub(crate) fn into_canonicalized(mut self) -> Self {
        self.authoritative_patch = self.authoritative_patch.into_canonicalized();
        self.semantic_changes
            .sort_by_key(super::PublishedAuthoritativeAspectChange::canonical_key);
        self.semantic_changes.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalAuthoritativePatch {
    pub ordering: PatchOrdering,
    pub publication_mode: PatchPublicationMode,
    pub authoritative_record_patches: Vec<PublishedAuthoritativeRecordPatch>,
}

impl CanonicalAuthoritativePatch {
    pub fn canonicalized(&self) -> Self {
        Self {
            ordering: self.ordering,
            publication_mode: self.publication_mode,
            authoritative_record_patches: self
                .authoritative_record_patches
                .iter()
                .map(PublishedAuthoritativeRecordPatch::canonicalized)
                .collect(),
        }
    }
}

/// Subscriber-facing projection of one canonical patch at its performed
/// runtime stream position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedAuthoritativePatchEnvelope {
    pub ordering: PatchOrdering,
    pub publication_mode: PatchPublicationMode,
    pub position: PatchStreamPosition,
    pub authoritative_record_patches: Vec<PublishedAuthoritativeRecordPatch>,
}

impl PublishedAuthoritativePatchEnvelope {
    pub(crate) fn from_canonical(
        position: PatchStreamPosition,
        patch: &CanonicalAuthoritativePatch,
    ) -> Self {
        Self {
            ordering: patch.ordering,
            publication_mode: patch.publication_mode,
            position,
            authoritative_record_patches: patch.authoritative_record_patches.clone(),
        }
    }

    pub fn canonicalized(&self) -> Self {
        let canonical = CanonicalAuthoritativePatch {
            ordering: self.ordering,
            publication_mode: self.publication_mode,
            authoritative_record_patches: self.authoritative_record_patches.clone(),
        }
        .canonicalized();
        Self::from_canonical(self.position, &canonical)
    }
}

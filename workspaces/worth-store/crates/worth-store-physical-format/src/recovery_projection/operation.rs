use crate::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, IndexedThroughBlobPublication,
    PersistedRecordIdentity, ReleaseCustodyHeadMutationV1, SelectedRecordContentClass,
};

use super::{
    PersistedDerivedDirectoryRetirement, PersistedReleaseCustodyHeadEffectV1,
    PersistedReleaseHeadTreeClaim, PersistedReleasedDirectoryReplacementV1,
    PersistedTerminalReleaseHeadRetirementV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedDerivedDirectoryRecordBinding {
    record: PersistedBlobSemanticRecordBinding,
    indexed_through: Option<IndexedThroughBlobPublication>,
    indexed_through_quarantine: Option<Option<PersistedRecordIdentity>>,
}

impl PersistedDerivedDirectoryRecordBinding {
    pub const fn new(
        record: PersistedBlobSemanticRecordBinding,
        indexed_through: Option<IndexedThroughBlobPublication>,
    ) -> Self {
        Self {
            record,
            indexed_through,
            indexed_through_quarantine: None,
        }
    }

    pub const fn new_with_quarantine(
        record: PersistedBlobSemanticRecordBinding,
        indexed_through: Option<IndexedThroughBlobPublication>,
        indexed_through_quarantine: Option<PersistedRecordIdentity>,
    ) -> Self {
        Self {
            record,
            indexed_through,
            indexed_through_quarantine: Some(indexed_through_quarantine),
        }
    }

    pub const fn record(self) -> PersistedBlobSemanticRecordBinding {
        self.record
    }
    pub const fn indexed_through(self) -> Option<IndexedThroughBlobPublication> {
        self.indexed_through
    }
    pub const fn indexed_through_quarantine(self) -> Option<Option<PersistedRecordIdentity>> {
        self.indexed_through_quarantine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersistedBlobSemanticRecordBinding {
    record: PersistedRecordIdentity,
    record_payload_sha256: [u8; 32],
    candidate_root_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedPhysicalRecoveryOperation {
    None,
    SessionDeclared(PersistedBlobSemanticRecordBinding),
    GenerationPublished(PersistedBlobSemanticRecordBinding),
    SessionFrontier(PersistedBlobSemanticRecordBinding),
    SessionAbandoned(PersistedBlobSemanticRecordBinding),
    RecordsDropped {
        binding: PersistedBlobSemanticRecordBinding,
        head_effect: Option<PersistedReleaseCustodyHeadEffectV1>,
        directory_replacement: Option<PersistedReleasedDirectoryReplacementV1>,
    },
    DerivedDirectory {
        binding: PersistedDerivedDirectoryRecordBinding,
        retirement: Option<PersistedDerivedDirectoryRetirement>,
    },
    ChunkReused(PersistedBlobSemanticRecordBinding),
    DedupeQuarantined(PersistedBlobSemanticRecordBinding),
    /// A terminal release head left the head tree. The only operation whose
    /// member carries no data record.
    TerminalReleaseHeadRetired(PersistedTerminalReleaseHeadRetirementV1),
}

impl PersistedBlobSemanticRecordBinding {
    pub fn new(
        record: PersistedRecordIdentity,
        record_payload_sha256: [u8; 32],
        candidate_root_generation: u64,
    ) -> Option<Self> {
        (candidate_root_generation != 0).then_some(Self {
            record,
            record_payload_sha256,
            candidate_root_generation,
        })
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }

    pub const fn record_payload_sha256(self) -> [u8; 32] {
        self.record_payload_sha256
    }

    pub const fn candidate_root_generation(self) -> u64 {
        self.candidate_root_generation
    }
}

impl PersistedPhysicalRecoveryOperation {
    pub const fn is_terminal_release_head_retired(&self) -> bool {
        matches!(self, Self::TerminalReleaseHeadRetired(_))
    }

    /// The head-tree frames this operation carries, if any. Exhaustive, so a
    /// new operation must state whether it owns such a claim.
    pub const fn release_head_tree_claim(&self) -> Option<PersistedReleaseHeadTreeClaim<'_>> {
        match self {
            Self::RecordsDropped {
                head_effect: Some(effect),
                ..
            } => Some(PersistedReleaseHeadTreeClaim::Upsert(effect)),
            Self::TerminalReleaseHeadRetired(retirement) => Some(
                PersistedReleaseHeadTreeClaim::TerminalHeadRetired(retirement),
            ),
            Self::None
            | Self::SessionDeclared(_)
            | Self::GenerationPublished(_)
            | Self::SessionFrontier(_)
            | Self::SessionAbandoned(_)
            | Self::RecordsDropped {
                head_effect: None, ..
            }
            | Self::DerivedDirectory { .. }
            | Self::ChunkReused(_)
            | Self::DedupeQuarantined(_) => None,
        }
    }

    pub(super) fn admits(
        &self,
        source_root_generation: u64,
        records: &[PersistedRecordIdentity],
        placements: &[CurrentPhysicalRecordPlacement],
    ) -> bool {
        let binding = match self {
            Self::None => return true,
            Self::TerminalReleaseHeadRetired(retirement) => {
                return placements.is_empty()
                    && retirement.source_root_generation() == source_root_generation;
            }
            Self::DerivedDirectory {
                binding,
                retirement,
            } => {
                return admits_classified_append(
                    binding.record(),
                    source_root_generation,
                    records,
                    placements,
                ) && retirement.as_ref().is_none_or(|retirement| {
                    !retirement
                        .dropped_records()
                        .iter()
                        .any(|record| records.contains(record))
                });
            }
            Self::ChunkReused(binding) | Self::DedupeQuarantined(binding) => {
                return admits_classified_append(
                    *binding,
                    source_root_generation,
                    records,
                    placements,
                );
            }
            Self::SessionDeclared(binding)
            | Self::GenerationPublished(binding)
            | Self::SessionFrontier(binding)
            | Self::SessionAbandoned(binding) => *binding,
            Self::RecordsDropped {
                binding,
                head_effect,
                directory_replacement,
            } => {
                if head_effect.as_ref().is_some_and(|effect| {
                    !matches!(effect.mutation(), ReleaseCustodyHeadMutationV1::Upsert { next, .. }
                        if next.descriptor_record() == binding.record()
                            && next.source_root_generation() == source_root_generation)
                }) {
                    return false;
                }
                if let Some(replacement) = directory_replacement {
                    let next = replacement.next().record();
                    let [descriptor, directory] = records else {
                        return false;
                    };
                    let [CurrentPhysicalRecordPlacement::Extent(first), CurrentPhysicalRecordPlacement::Extent(second)] =
                        placements
                    else {
                        return false;
                    };
                    let a = first.arena_range();
                    let b = second.arena_range();
                    return head_effect.is_some()
                        && *descriptor == binding.record()
                        && *directory == next.record()
                        && first.record() == *descriptor
                        && second.record() == *directory
                        && first.content_class()
                            == SelectedRecordContentClass::Blob(
                                BlobRecordKind::ReclaimDescriptorV3,
                            )
                        && second.content_class() == SelectedRecordContentClass::DerivedDirectory
                        && first.extent() != second.extent()
                        && (a.arena() != b.arena()
                            || a.end() <= b.offset()
                            || b.end() <= a.offset())
                        && source_root_generation.checked_add(1)
                            == Some(binding.candidate_root_generation())
                        && next.candidate_root_generation() == binding.candidate_root_generation();
                }
                *binding
            }
        };
        records == [binding.record()]
            && matches!(
                placements,
                [CurrentPhysicalRecordPlacement::Extent(extent)]
                    if extent.record() == binding.record()
            )
            && source_root_generation.checked_add(1) == Some(binding.candidate_root_generation())
    }
}

fn admits_classified_append(
    binding: PersistedBlobSemanticRecordBinding,
    source_root_generation: u64,
    records: &[PersistedRecordIdentity],
    placements: &[CurrentPhysicalRecordPlacement],
) -> bool {
    records == [binding.record()]
        && placements
            .iter()
            .any(|placement| placement.record() == binding.record())
        && source_root_generation.checked_add(1) == Some(binding.candidate_root_generation())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DurableInlineRecordPlacement, PhysicalGeneration, PhysicalGenerationAuthority,
        PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId,
    };

    #[test]
    fn reused_chunk_claim_admits_inline_placement_and_requires_selected_target() {
        let record = PersistedRecordIdentity::new([1; 16], 2).unwrap();
        let other = PersistedRecordIdentity::new([1; 16], 3).unwrap();
        let binding = PersistedBlobSemanticRecordBinding::new(record, [4; 32], 11).unwrap();
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let segment = authority
            .segment_cell(PhysicalSegmentId::from_raw(1).unwrap())
            .with_segment_generation(PhysicalGeneration::from_raw(10).unwrap());
        let page = authority
            .page_cell(
                PhysicalSegmentId::from_raw(1).unwrap(),
                PhysicalPageId::from_raw(1).unwrap(),
            )
            .with_page_generation(PhysicalGeneration::from_raw(10).unwrap());
        let slot = authority
            .slot_cell(
                PhysicalSegmentId::from_raw(1).unwrap(),
                PhysicalPageId::from_raw(1).unwrap(),
                PhysicalRecordSlot::from_raw(1).unwrap(),
            )
            .with_slot_generation(PhysicalGeneration::from_raw(10).unwrap());
        let placement = CurrentPhysicalRecordPlacement::Inline(
            DurableInlineRecordPlacement::legacy_unknown(record, segment, page, slot, 4096, 216)
                .unwrap(),
        );
        let semantic = PersistedPhysicalRecoveryOperation::ChunkReused(binding);
        assert!(semantic.admits(10, &[record], &[placement]));
        assert!(!semantic.admits(10, &[other], &[placement]));
        assert!(!semantic.admits(9, &[record], &[placement]));
    }
}

use worth_store_physical_format::{
    PageGenerationCell, PhysicalGeneration, PhysicalGenerationOwner, PhysicalReference,
    PhysicalReferenceAdmissionWitness, SegmentGenerationCell,
};

use super::{
    mismatch_for_page, mismatch_for_record_extent, mismatch_for_reference, mismatch_for_segment,
};
use crate::{GenerationCountedReferenceDenial, PhysicalReferenceGenerationMismatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationCountedPhysicalReference {
    AdmittedReference(PhysicalReference),
    RecordExtent { owner: PhysicalGenerationOwner },
    Segment { owner: PhysicalGenerationOwner },
    Page { owner: PhysicalGenerationOwner },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentGenerationPhysicalReference {
    reference: GenerationCountedPhysicalReference,
}

impl GenerationCountedPhysicalReference {
    pub const fn from_admitted_reference(admission: PhysicalReferenceAdmissionWitness) -> Self {
        Self::AdmittedReference(admission.reference())
    }

    pub const fn from_segment_cell(cell: SegmentGenerationCell) -> Self {
        Self::Segment {
            owner: cell.owner(),
        }
    }

    pub const fn from_page_cell(cell: PageGenerationCell) -> Self {
        Self::Page {
            owner: cell.owner(),
        }
    }

    pub fn require_current_generation(
        self,
        observed_generation: PhysicalGeneration,
    ) -> Result<CurrentGenerationPhysicalReference, PhysicalReferenceGenerationMismatch> {
        match self {
            Self::AdmittedReference(reference) => {
                if reference.generation() == observed_generation {
                    Ok(CurrentGenerationPhysicalReference::from_validated_reference(self))
                } else {
                    Err(mismatch_for_reference(reference, observed_generation))
                }
            }
            Self::Segment { owner } => {
                if owner.generation() == observed_generation {
                    Ok(CurrentGenerationPhysicalReference::from_validated_reference(self))
                } else {
                    Err(mismatch_for_segment(
                        owner.generation(),
                        observed_generation,
                    ))
                }
            }
            Self::RecordExtent { owner } => {
                if owner.generation() == observed_generation {
                    Ok(CurrentGenerationPhysicalReference::from_validated_reference(self))
                } else {
                    Err(mismatch_for_record_extent(
                        owner.generation(),
                        observed_generation,
                    ))
                }
            }
            Self::Page { owner } => {
                if owner.generation() == observed_generation {
                    Ok(CurrentGenerationPhysicalReference::from_validated_reference(self))
                } else {
                    Err(mismatch_for_page(owner.generation(), observed_generation))
                }
            }
        }
    }

    pub const fn generation(self) -> PhysicalGeneration {
        match self {
            Self::AdmittedReference(reference) => reference.generation(),
            Self::RecordExtent { owner } | Self::Segment { owner } | Self::Page { owner } => {
                owner.generation()
            }
        }
    }

    pub fn owner(self) -> PhysicalGenerationOwner {
        match self {
            Self::AdmittedReference(reference) => reference.generation_owner(),
            Self::RecordExtent { owner } | Self::Segment { owner } | Self::Page { owner } => owner,
        }
    }

    pub const fn reject_future_chunk_lifecycle_claim() -> GenerationCountedReferenceDenial {
        GenerationCountedReferenceDenial::FutureChunkLifecycleNotOwnedByS5
    }
}

impl CurrentGenerationPhysicalReference {
    const fn from_validated_reference(reference: GenerationCountedPhysicalReference) -> Self {
        Self { reference }
    }

    pub const fn generation(self) -> PhysicalGeneration {
        self.reference.generation()
    }

    pub fn owner(self) -> PhysicalGenerationOwner {
        self.reference.owner()
    }

    pub(crate) const fn generation_counted_reference(self) -> GenerationCountedPhysicalReference {
        self.reference
    }

    pub(crate) fn from_durable_owner(owner: PhysicalGenerationOwner) -> Option<Self> {
        GenerationCountedPhysicalReference::from_durable_owner(owner)?
            .require_current_generation(owner.generation())
            .ok()
    }
}

impl GenerationCountedPhysicalReference {
    /// Describe a canonical durable owner; this does not grant live runtime authority.
    pub fn from_durable_owner(owner: PhysicalGenerationOwner) -> Option<Self> {
        use worth_store_physical_format::{
            PhysicalCellReuseDomain, PhysicalGenerationAuthority, PhysicalReferenceAuthority,
        };

        let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
        let references = PhysicalReferenceAuthority::for_canonical_physical_format();
        let counted = match owner.domain() {
            PhysicalCellReuseDomain::SlotAllocation => {
                let cell = generations
                    .slot_cell(owner.segment_id()?, owner.page_id()?, owner.slot()?)
                    .with_slot_generation(owner.generation());
                GenerationCountedPhysicalReference::from_admitted_reference(
                    references.admit_page_slot(cell),
                )
            }
            PhysicalCellReuseDomain::ExtentAllocation => {
                let cell = generations
                    .extent_cell(owner.segment_id()?, owner.extent_id()?)
                    .with_extent_generation(owner.generation());
                GenerationCountedPhysicalReference::from_admitted_reference(
                    references.admit_extent(cell),
                )
            }
            PhysicalCellReuseDomain::RecordExtentAllocation => {
                GenerationCountedPhysicalReference::RecordExtent { owner }
            }
            PhysicalCellReuseDomain::RootPublication => {
                let cell = generations
                    .root_publication_cell(owner.root_reference()?)
                    .with_root_publication_generation(owner.generation());
                GenerationCountedPhysicalReference::from_admitted_reference(
                    references.admit_root_publication(cell),
                )
            }
            PhysicalCellReuseDomain::Page => {
                let cell = generations
                    .page_cell(owner.segment_id()?, owner.page_id()?)
                    .with_page_generation(owner.generation());
                GenerationCountedPhysicalReference::from_page_cell(cell)
            }
            PhysicalCellReuseDomain::Segment => {
                let cell = generations
                    .segment_cell(owner.segment_id()?)
                    .with_segment_generation(owner.generation());
                GenerationCountedPhysicalReference::from_segment_cell(cell)
            }
            PhysicalCellReuseDomain::FreeSpaceReuse => return None,
        };
        Some(counted)
    }
}

use worth_proof::{CanonicalVec, NonEmpty, UniqueVec};
use worth_store_physical_format::{PhysicalCellReuseDomain, PhysicalGenerationOwner};

use super::{
    PhysicalReadPlanAdmissionDenial, ProtectedReferenceRange, ProtectedReferenceRangeSet,
    ReadPlanAdmissionScratchArena, ReadPlanScratchUsage,
};
use crate::{CurrentGenerationPhysicalReference, GenerationCountedPhysicalReference};

#[cfg(any(test, feature = "certification-authority"))]
mod footprint_test_authority;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectedPhysicalReference {
    reference: CurrentGenerationPhysicalReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedPhysicalReferenceSet {
    non_empty: NonEmpty<ProtectedPhysicalReference>,
    unique: UniqueVec<ProtectedPhysicalReference>,
    canonical: CanonicalVec<ProtectedPhysicalReference>,
    range_scratch: Option<Vec<ProtectedReferenceRange>>,
    scratch_usage: Option<ReadPlanScratchUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactProtectedReferenceSet {
    non_empty: NonEmpty<ProtectedPhysicalReference>,
    unique: UniqueVec<ProtectedPhysicalReference>,
    canonical: CanonicalVec<ProtectedPhysicalReference>,
    ranges: ProtectedReferenceRangeSet,
    scratch_usage: ReadPlanScratchUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReadProtectedFootprintBasis {
    protected_references: u64,
    protected_ranges: u64,
    canonical_digest: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalReferenceSortKey {
    domain: PhysicalCellReuseDomain,
    segment_id: Option<u64>,
    extent_id: Option<u64>,
    page_id: Option<u64>,
    slot: Option<u16>,
    root_reference: Option<u64>,
    allocation_class: Option<worth_store_physical_format::AllocationClassKind>,
    generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalReadPlanFootprint {
    protected: CompactProtectedReferenceSet,
    resident_bytes: u64,
}

impl ProtectedPhysicalReference {
    pub const fn from_current_generation(reference: CurrentGenerationPhysicalReference) -> Self {
        Self { reference }
    }

    pub const fn current_generation(self) -> CurrentGenerationPhysicalReference {
        self.reference
    }

    pub fn owner(self) -> PhysicalGenerationOwner {
        self.reference.owner()
    }

    pub fn sort_key(self) -> PhysicalReferenceSortKey {
        PhysicalReferenceSortKey::from_owner(self.owner())
    }
}

impl Ord for ProtectedPhysicalReference {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl PartialOrd for ProtectedPhysicalReference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ProtectedPhysicalReferenceSet {
    pub fn from_current_generation_refs(
        references: impl IntoIterator<Item = CurrentGenerationPhysicalReference>,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        let protected = references
            .into_iter()
            .map(ProtectedPhysicalReference::from_current_generation)
            .collect::<Vec<_>>();
        Self::from_protected_references(protected)
    }

    pub fn from_current_generation_refs_with_scratch<I>(
        references: I,
        scratch: ReadPlanAdmissionScratchArena,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial>
    where
        I: IntoIterator<Item = CurrentGenerationPhysicalReference>,
        I::IntoIter: ExactSizeIterator,
    {
        let buffers = scratch.protect_current_generation_refs(references)?;
        Self::from_protected_references_with_usage(
            buffers.references,
            Some(buffers.ranges),
            buffers.usage,
        )
    }

    pub fn from_generation_counted_refs(
        references: impl IntoIterator<
            Item = (
                GenerationCountedPhysicalReference,
                worth_store_physical_format::PhysicalGeneration,
            ),
        >,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        let mut protected = Vec::new();
        for (reference, observed_generation) in references {
            protected.push(ProtectedPhysicalReference::from_current_generation(
                reference
                    .require_current_generation(observed_generation)
                    .map_err(PhysicalReadPlanAdmissionDenial::StaleGeneration)?,
            ));
        }
        Self::from_protected_references(protected)
    }

    pub fn from_protected_references(
        mut references: Vec<ProtectedPhysicalReference>,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        Self::from_protected_references_with_optional_usage(&mut references, None, None)
    }

    fn from_protected_references_with_usage(
        mut references: Vec<ProtectedPhysicalReference>,
        range_scratch: Option<Vec<ProtectedReferenceRange>>,
        usage: ReadPlanScratchUsage,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        Self::from_protected_references_with_optional_usage(
            &mut references,
            range_scratch,
            Some(usage.with_proof_wrapper_construction()),
        )
    }

    fn from_protected_references_with_optional_usage(
        references: &mut Vec<ProtectedPhysicalReference>,
        range_scratch: Option<Vec<ProtectedReferenceRange>>,
        scratch_usage: Option<ReadPlanScratchUsage>,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        if references.is_empty() {
            return Err(PhysicalReadPlanAdmissionDenial::EmptyProtectedFootprint);
        }
        references.sort();
        references.dedup();
        let non_empty = NonEmpty::try_from_vec(references.clone())
            .map_err(|_| PhysicalReadPlanAdmissionDenial::EmptyProtectedFootprint)?;
        let unique = UniqueVec::try_from_unique(references.clone())
            .map_err(|_| PhysicalReadPlanAdmissionDenial::EmptyProtectedFootprint)?;
        let canonical = CanonicalVec::try_from_sorted(references.clone())
            .map_err(|_| PhysicalReadPlanAdmissionDenial::EmptyProtectedFootprint)?;
        Ok(Self {
            non_empty,
            unique,
            canonical,
            range_scratch,
            scratch_usage,
        })
    }

    pub fn references(&self) -> &[ProtectedPhysicalReference] {
        self.canonical.as_slice()
    }

    pub const fn non_empty(&self) -> &NonEmpty<ProtectedPhysicalReference> {
        &self.non_empty
    }

    pub const fn unique(&self) -> &UniqueVec<ProtectedPhysicalReference> {
        &self.unique
    }

    pub const fn canonical(&self) -> &CanonicalVec<ProtectedPhysicalReference> {
        &self.canonical
    }

    pub const fn scratch_usage(&self) -> Option<ReadPlanScratchUsage> {
        self.scratch_usage
    }

    pub fn footprint_basis(&self) -> PhysicalReadProtectedFootprintBasis {
        PhysicalReadProtectedFootprintBasis::from_references(self.references())
    }
}

impl CompactProtectedReferenceSet {
    pub fn from_reference_set_with_scratch(
        set: ProtectedPhysicalReferenceSet,
        scratch: ReadPlanAdmissionScratchArena,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        if set.scratch_usage().is_none() {
            return Err(
                PhysicalReadPlanAdmissionDenial::UnboundedProtectedFootprint {
                    requested: set.references().len(),
                    capacity: 0,
                },
            );
        }
        let ProtectedPhysicalReferenceSet {
            non_empty,
            unique,
            canonical,
            ..
        } = set;
        let buffers = scratch.protect_existing_refs(canonical.as_slice().iter().copied())?;
        let ranges =
            ProtectedReferenceRangeSet::from_references(&buffers.references, buffers.ranges);
        Ok(Self {
            non_empty,
            unique,
            canonical,
            ranges,
            scratch_usage: buffers.usage.with_range_compaction(),
        })
    }

    pub fn references(&self) -> &[ProtectedPhysicalReference] {
        self.canonical.as_slice()
    }

    pub const fn non_empty(&self) -> &NonEmpty<ProtectedPhysicalReference> {
        &self.non_empty
    }

    pub const fn unique(&self) -> &UniqueVec<ProtectedPhysicalReference> {
        &self.unique
    }

    pub const fn canonical(&self) -> &CanonicalVec<ProtectedPhysicalReference> {
        &self.canonical
    }

    pub const fn ranges(&self) -> &ProtectedReferenceRangeSet {
        &self.ranges
    }

    pub const fn scratch_usage(&self) -> ReadPlanScratchUsage {
        self.scratch_usage
    }

    pub fn footprint_basis(&self) -> PhysicalReadProtectedFootprintBasis {
        PhysicalReadProtectedFootprintBasis::from_compact_set(self)
    }

    pub fn declared_footprint_basis(&self) -> PhysicalReadProtectedFootprintBasis {
        PhysicalReadProtectedFootprintBasis::from_references(self.references())
    }

    pub fn contains_current_generation(
        &self,
        reference: CurrentGenerationPhysicalReference,
    ) -> bool {
        self.ranges
            .contains_reference(ProtectedPhysicalReference::from_current_generation(
                reference,
            ))
    }
}

impl PhysicalReadPlanFootprint {
    pub(crate) fn new(protected: CompactProtectedReferenceSet, resident_bytes: u64) -> Self {
        Self {
            protected,
            resident_bytes,
        }
    }

    pub const fn protected(&self) -> &CompactProtectedReferenceSet {
        &self.protected
    }

    pub const fn resident_bytes(&self) -> u64 {
        self.resident_bytes
    }

    pub fn footprint_basis(&self) -> PhysicalReadProtectedFootprintBasis {
        self.protected.footprint_basis()
    }

    pub fn declared_footprint_basis(&self) -> PhysicalReadProtectedFootprintBasis {
        self.protected.declared_footprint_basis()
    }

    pub fn admits_reference(&self, reference: CurrentGenerationPhysicalReference) -> bool {
        self.protected.contains_current_generation(reference)
    }
}

impl PhysicalReadProtectedFootprintBasis {
    pub(crate) fn from_references(references: &[ProtectedPhysicalReference]) -> Self {
        Self {
            protected_references: references.len() as u64,
            protected_ranges: 0,
            canonical_digest: canonical_footprint_digest(references, &[]),
        }
    }

    fn from_compact_set(set: &CompactProtectedReferenceSet) -> Self {
        Self {
            protected_references: set.references().len() as u64,
            protected_ranges: set.ranges().ranges().len() as u64,
            canonical_digest: canonical_footprint_digest(set.references(), set.ranges().ranges()),
        }
    }

    pub const fn protected_references(self) -> u64 {
        self.protected_references
    }

    pub const fn protected_ranges(self) -> u64 {
        self.protected_ranges
    }

    pub const fn canonical_digest(self) -> u64 {
        self.canonical_digest
    }
}

impl PhysicalReferenceSortKey {
    fn from_owner(owner: PhysicalGenerationOwner) -> Self {
        Self {
            domain: owner.domain(),
            segment_id: owner.segment_id().map(|segment| segment.get()),
            extent_id: owner.extent_id().map(|extent| extent.get()),
            page_id: owner.page_id().map(|page| page.get()),
            slot: owner.slot().map(|slot| slot.get()),
            root_reference: owner.root_reference().map(|root| root.get()),
            allocation_class: owner.allocation_class(),
            generation: owner.generation().get(),
        }
    }
}

fn canonical_footprint_digest(
    references: &[ProtectedPhysicalReference],
    ranges: &[ProtectedReferenceRange],
) -> u64 {
    let mut digest = 0xcbf29ce484222325_u64;
    for reference in references {
        mix_u64(&mut digest, reference.sort_key().digest_component());
    }
    for range in ranges {
        mix_u64(&mut digest, range.digest_component());
    }
    digest
}

impl PhysicalReferenceSortKey {
    fn digest_component(self) -> u64 {
        let mut digest = 0xcbf29ce484222325_u64;
        mix_u64(&mut digest, self.domain as u64);
        mix_optional_u64(&mut digest, self.segment_id);
        mix_optional_u64(&mut digest, self.extent_id);
        mix_optional_u64(&mut digest, self.page_id);
        mix_optional_u64(&mut digest, self.slot.map(u64::from));
        mix_optional_u64(&mut digest, self.root_reference);
        mix_u64(
            &mut digest,
            self.allocation_class
                .map(|class| class as u64)
                .unwrap_or(u64::MAX),
        );
        mix_u64(&mut digest, self.generation);
        digest
    }
}

fn mix_optional_u64(digest: &mut u64, value: Option<u64>) {
    mix_u64(digest, value.unwrap_or(u64::MAX));
}

fn mix_u64(digest: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *digest ^= u64::from(byte);
        *digest = digest.wrapping_mul(0x100000001b3);
    }
}

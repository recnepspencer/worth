use super::{CompactionProtectedReferenceSet, CompactionReadInterlockCounters};
use crate::{
    CurrentGenerationPhysicalReference, PhysicalReadPlanAdmissionDenial,
    ProtectedPhysicalReference, ProtectedReferenceRange, ProtectedReferenceRangeSet,
};
use worth_store_physical_format::PhysicalGenerationOwner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionCandidateRangeSet {
    footprint_basis: crate::PhysicalReadProtectedFootprintBasis,
    ranges: ProtectedReferenceRangeSet,
    references: Vec<CurrentGenerationPhysicalReference>,
    owners: Vec<PhysicalGenerationOwner>,
    candidate_references: u64,
}

impl CompactionCandidateRangeSet {
    pub fn from_current_generation_refs(
        references: impl IntoIterator<Item = CurrentGenerationPhysicalReference>,
    ) -> Result<Self, PhysicalReadPlanAdmissionDenial> {
        let mut protected = references
            .into_iter()
            .map(ProtectedPhysicalReference::from_current_generation)
            .collect::<Vec<_>>();
        if protected.is_empty() {
            return Err(PhysicalReadPlanAdmissionDenial::EmptyProtectedFootprint);
        }
        protected.sort();
        protected.dedup();
        let owners = protected
            .iter()
            .map(|reference| reference.owner())
            .collect::<Vec<_>>();
        let references = protected
            .iter()
            .map(|reference| reference.current_generation())
            .collect::<Vec<_>>();
        let ranges = ProtectedReferenceRangeSet::from_references(
            &protected,
            Vec::<ProtectedReferenceRange>::with_capacity(protected.len()),
        );
        Ok(Self {
            footprint_basis: crate::PhysicalReadProtectedFootprintBasis::from_references(
                &protected,
            ),
            ranges,
            references,
            owners,
            candidate_references: protected.len() as u64,
        })
    }

    pub fn intersect_protected(
        &self,
        protected: &CompactionProtectedReferenceSet,
    ) -> CompactionReadInterlockCounters {
        let intersection = protected
            .ranges()
            .bounded_intersection(self.ranges.ranges());
        CompactionReadInterlockCounters::from_range_intersection(
            intersection.protected_ranges(),
            intersection.candidate_ranges(),
            intersection.range_comparisons(),
            intersection.overlapping_ranges(),
            self.candidate_references,
        )
    }

    pub const fn ranges(&self) -> &ProtectedReferenceRangeSet {
        &self.ranges
    }

    pub const fn footprint_basis(&self) -> crate::PhysicalReadProtectedFootprintBasis {
        self.footprint_basis
    }

    pub fn references(&self) -> &[CurrentGenerationPhysicalReference] {
        &self.references
    }

    pub fn is_fully_covered_by_owner(&self, owner: PhysicalGenerationOwner) -> bool {
        self.owners
            .iter()
            .all(|candidate_owner| *candidate_owner == owner)
    }

    pub const fn candidate_references(&self) -> u64 {
        self.candidate_references
    }
}

//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn by_source_for_test(
        &self,
    ) -> &BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    > {
        &self.by_source
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn by_source_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    > {
        &mut self.by_source
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn new_for_test(
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Self {
        Self {
            occurrence,
            generation,
        }
    }
}

impl WorthQueryCurrentOutputFamilyResolution {
    #[doc(hidden)]
    pub fn ambiguous_publication_for_test(&self) -> &bool {
        &self.ambiguous_publication
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn binding_identities_for_test(
        &self,
    ) -> &super::super::output_binding_identity::OutputBindingInventory {
        &self.binding_identities
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn cancelled_slots_for_test(&self) -> &Option<Box<prepared_slot::CancelledLineageSlot>> {
        &self.cancelled_slots
    }
}

impl WorthQueryCurrentOutputFamilyResolution {
    #[doc(hidden)]
    pub fn candidates_for_test(&self) -> &Vec<WorthQueryCurrentOutputCandidate> {
        &self.candidates
    }
}

impl WorthQueryPriorOutputBindingResolution {
    #[doc(hidden)]
    pub fn correspondence_for_test(
        &self,
    ) -> &Option<Arc<WorthQueryApplicationOutputCorrespondence>> {
        &self.correspondence
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn generation_for_test(&self) -> &u64 {
        &self.generation
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn occurrence_for_test(&self) -> &worth_runtime_world::facade::ProductBranchIncarnation {
        &self.occurrence
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn origins_for_test(
        &self,
    ) -> &BTreeMap<worth_runtime_world::facade::ProductBranchIncarnation, ProductCoordinate> {
        &self.origins
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn output_families_for_test(
        &self,
    ) -> &BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>> {
        &self.output_families
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn partition_index_for_test(&self) -> &partition_index::OutputPartitionIndex {
        &self.partition_index
    }
}

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn generation_mut_for_test(&mut self) -> &mut u64 {
        &mut self.generation
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn partition_index_mut_for_test(&mut self) -> &mut partition_index::OutputPartitionIndex {
        &mut self.partition_index
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn install_output_families_for_test(
        &mut self,
        families: BTreeMap<OutputFamilyIdentity, Vec<(TypeId, String)>>,
    ) {
        Self::install_output_families(self, families)
    }
}

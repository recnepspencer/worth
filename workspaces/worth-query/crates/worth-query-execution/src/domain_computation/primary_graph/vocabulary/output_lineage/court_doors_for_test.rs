//! Sealed test doors over this module's private members.
use crate::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;

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

impl ProductCoordinate {
    #[doc(hidden)]
    pub fn generation_mut_for_test(&mut self) -> &mut u64 {
        &mut self.generation
    }
}

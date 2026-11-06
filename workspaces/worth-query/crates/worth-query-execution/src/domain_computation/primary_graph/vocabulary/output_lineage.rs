//! `output_lineage`'s vocabulary, moved whole from `primary_graph::output_lineage`: its root file's items here, one child per other source file.
pub(crate) mod commit_touches;
pub(crate) mod derived;
pub(crate) mod mark_state;
pub(crate) mod semantic_source;
pub(crate) mod settlement_identity;
pub(crate) mod verification;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductCoordinate {
    pub(in crate::domain_computation::primary_graph) occurrence:
        worth_runtime_world::facade::ProductBranchIncarnation,
    pub(in crate::domain_computation::primary_graph) generation: u64,
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

mod court_doors_for_test;

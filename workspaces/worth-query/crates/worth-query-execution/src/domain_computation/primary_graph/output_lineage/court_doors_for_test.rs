//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn by_source_for_test(&self) -> &BTreeMap<
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
    pub fn by_source_mut_for_test(&mut self) -> &mut BTreeMap<
        SemanticSource,
        BTreeMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedGeneration>,
        >,
    > {
        &mut self.by_source
    }
}

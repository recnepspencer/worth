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

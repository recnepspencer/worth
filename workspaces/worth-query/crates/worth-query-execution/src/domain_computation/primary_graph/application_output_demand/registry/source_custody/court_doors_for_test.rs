//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn bind_prepared_output_source_for_test(
        &self,
        commit: &CompositeCommitIdentity,
        source: BoundOutputSource,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::bind_prepared_output_source(self, commit, source)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn ensure_discovered_sources_bound_for_test(
        &self,
        commit: &CompositeCommitIdentity,
        sources: &[BoundOutputSource],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::ensure_discovered_sources_bound(self, commit, sources)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn ensure_prepared_output_source_bound_for_test(
        &self,
        commit: &CompositeCommitIdentity,
        source: BoundOutputSource,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::ensure_prepared_output_source_bound(self, commit, source)
    }
}

impl SourceCustody {
    #[doc(hidden)]
    pub fn retire_superseded_roots_for_test(&mut self, successors: &[BoundOutputSource]) {
        Self::retire_superseded_roots(self, successors)
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn validate_recovery_root_kind_for_test(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        kind: PreparedOutputRootKind,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::validate_recovery_root_kind(self, &receipt.into(), kind)
    }
}

impl SourcePreparationState {
    #[doc(hidden)]
    pub fn retired_for_test(&self) -> &bool {
        &self.retired
    }
}

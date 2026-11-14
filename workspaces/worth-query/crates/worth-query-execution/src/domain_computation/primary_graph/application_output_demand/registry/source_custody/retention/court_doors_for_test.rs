//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn retain_performed_source_for_test(
        &self,
        source: WorthQueryRetainedOutputDemandSource,
        preparation: &super::super::super::WorthQueryRequiredOutputSourcePreparation,
        root_kind: PreparedOutputRootKind,
        discovery: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<CompositeCommitIdentity, WorthQueryOutputDemandDenial> {
        Self::retain_performed_source(self, source, preparation, root_kind, discovery, None)
    }
}

//! A Home source supplies its own actual Native handle and existing registries.
use super::*;
use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;

impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn attach_home_source(
        source: RuntimeBridgeRelationalSource,
        receipt: &RelationalInitialSchemaInstallationReceipt,
        invalidation_resources: super::super::super::WorthQueryInvalidationResources,
    ) -> Result<Self, HomeSourceAttachmentDenial> {
        let runtime = source.query_runtime_handle();
        let runtime_instance_id = runtime.runtime_instance_id();
        if runtime_instance_id != receipt.runtime_instance_id() {
            return Err(HomeSourceAttachmentDenial::ForeignInstalledNative);
        }
        // Only after authentic installed-Native identity matches may Query
        // activate a publication companion or lend any bootstrap effect lane.
        Self::from_registered_handle(runtime, source, runtime_instance_id, invalidation_resources)
            .map_err(HomeSourceAttachmentDenial::Installation)
    }
}

#[derive(Debug)]
pub(in crate::domain_computation) enum HomeSourceAttachmentDenial {
    ForeignInstalledNative,
    Installation(WorthQueryRelationalSourceInstallationDenial),
}

//! The concrete storage of one genuine installed operation.
use super::*;

#[derive(Clone)]
pub(super) struct InstalledOperationBacking {
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) owner: String,
    pub(super) schema_name: String,
    pub(super) operation: String,
    pub(super) input_type: String,
    pub(super) contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
    pub(super) native_contracts:
        Arc<crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog>,
    pub(super) portable_native_contracts: Arc<Vec<WorthQueryPortableNativeAspectContractRecord>>,
    pub(super) portable_contract: WorthQueryPortableApplicationOperationContractRecord,
    pub(super) obligations: WorthQueryInstalledGraphObligationSet,
    pub(super) authority_identity: AuthoritySeal,
    pub(super) definition_identity: [u8; 32],
    pub(super) operation_type_id: std::any::TypeId,
    pub(super) input_type_id: std::any::TypeId,
}

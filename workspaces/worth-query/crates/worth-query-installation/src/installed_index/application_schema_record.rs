use std::sync::Arc;

use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration;

use crate::application_schema::WorthQueryInstalledApplicationSchemaContractCatalog;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::package::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableNativeAspectContractRecord,
};

#[derive(Debug)]
pub(super) struct WorthQueryInstalledApplicationSchemaRecord {
    operation_catalog: Arc<crate::application_operation::InstalledOperationCatalog>,
    declaration: ErasedApplicationSchemaDeclaration,
    schema_identity: CanonicalDigestId,
    schema_work: WorthQueryCanonicalWorkEvidence,
    catalog: Arc<WorthQueryInstalledApplicationSchemaContractCatalog>,
    native_contracts: Arc<Vec<WorthQueryPortableNativeAspectContractRecord>>,
    operation_contracts: Arc<Vec<WorthQueryPortableApplicationOperationContractRecord>>,
}

impl WorthQueryInstalledApplicationSchemaRecord {
    pub(super) fn new(
        declaration: ErasedApplicationSchemaDeclaration,
        schema_identity: CanonicalDigestId,
        schema_work: WorthQueryCanonicalWorkEvidence,
        catalog: Arc<WorthQueryInstalledApplicationSchemaContractCatalog>,
        native_contracts: Arc<Vec<WorthQueryPortableNativeAspectContractRecord>>,
        operation_contracts: Arc<Vec<WorthQueryPortableApplicationOperationContractRecord>>,
    ) -> Self {
        Self {
            operation_catalog: Default::default(),
            declaration,
            schema_identity,
            schema_work,
            catalog,
            native_contracts,
            operation_contracts,
        }
    }

    pub(super) fn declaration(&self) -> &ErasedApplicationSchemaDeclaration {
        &self.declaration
    }

    pub(super) fn operation_catalog(
        &self,
    ) -> Arc<crate::application_operation::InstalledOperationCatalog> {
        Arc::clone(&self.operation_catalog)
    }

    pub(super) const fn schema_identity(&self) -> CanonicalDigestId {
        self.schema_identity
    }

    pub(super) fn catalog(&self) -> &Arc<WorthQueryInstalledApplicationSchemaContractCatalog> {
        &self.catalog
    }

    pub(super) fn native_contracts(
        &self,
    ) -> &Arc<Vec<WorthQueryPortableNativeAspectContractRecord>> {
        &self.native_contracts
    }

    pub(super) fn operation_contracts(
        &self,
    ) -> &Arc<Vec<WorthQueryPortableApplicationOperationContractRecord>> {
        &self.operation_contracts
    }

    pub(super) fn installation_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.schema_work.combine(self.catalog.canonical_work())
    }
}

//! Borrow existing installed schema backings without a second schema form.
use super::*;
use crate::application_operation::InstalledOperationCatalog;

#[doc(hidden)]
pub struct InstalledSchemaView<'a> {
    binding: ApplicationSchemaBindingIdentity,
    declaration: &'a ErasedApplicationSchemaDeclaration,
    native: &'a Arc<WorthQueryInstalledApplicationSchemaContractCatalog>,
    operations: &'a Arc<InstalledOperationCatalog>,
}

impl<'a> InstalledSchemaView<'a> {
    fn new(
        binding: ApplicationSchemaBindingIdentity,
        declaration: &'a ErasedApplicationSchemaDeclaration,
        native: &'a Arc<WorthQueryInstalledApplicationSchemaContractCatalog>,
        operations: &'a Arc<InstalledOperationCatalog>,
    ) -> Self {
        Self {
            binding,
            declaration,
            native,
            operations,
        }
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }
    pub fn installed_declaration(&self) -> &ErasedApplicationSchemaDeclaration {
        self.declaration
    }
    pub fn native_contracts(&self) -> &WorthQueryInstalledApplicationSchemaContractCatalog {
        self.native
    }
    pub fn retain_operation_catalog(&self) -> Arc<InstalledOperationCatalog> {
        Arc::clone(self.operations)
    }
}

impl<S: ApplicationSchema> WorthQueryInstalledApplicationSchema<S> {
    #[doc(hidden)]
    pub fn installed_view(&self) -> InstalledSchemaView<'_> {
        InstalledSchemaView::new(
            self.binding_identity(),
            &self.schema,
            &self.native_contract_catalog,
            &self.operation_catalog,
        )
    }
}

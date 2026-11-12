//! Re-entry compares admission facts with the installation owner's retained handle.
use super::brand_tag::BrandTag;
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial as Denial,
    WorthQueryOperationAuthorizationDenialKind as Kind,
    WorthQueryOperationAuthorizationRefusalKind as Refusal,
};
use crate::equality_type_id::EqualityTypeId;
use worth_query_installation::facade::{InstalledOperationCatalog, InstalledOperationView};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[inline]
    pub(in crate::domain_computation) fn validate_installed_entry(
        &self,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        self.core
            .validate_installed_entry_with_tag(&self.tag, runtime, binding, installed)
    }

    #[inline]
    pub(in crate::domain_computation) fn validate_mutation_entry(
        &self,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), crate::domain_computation::primary_graph::MutationHandlerExecutionDenial> {
        self.core
            .validate_mutation_entry_with_tag(&self.tag, runtime, binding, installed)
    }

    #[inline]
    pub(in crate::domain_computation) fn validate_installed_operation(
        &self,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        self.core.validate_installed_operation(&self.tag, installed)
    }
}

impl WorthQueryAdmittedOperation {
    pub(in crate::domain_computation) fn validate_owned_brand(
        &self,
        tag: &BrandTag,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    ) -> Result<(), Denial> {
        self.validate_runtime_ownership(runtime, binding)?;
        tag.admits(self)
    }

    pub(in crate::domain_computation) fn validate_installed_entry_with_tag(
        &self,
        tag: &BrandTag,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        self.validate_owned_brand(tag, runtime, binding)?;
        self.validate_installed_operation(tag, installed)
    }

    fn validate_mutation_entry_with_tag(
        &self,
        tag: &BrandTag,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), crate::domain_computation::primary_graph::MutationHandlerExecutionDenial> {
        self.validate_installed_entry_with_tag(tag, runtime, binding, installed)
            .map_err(mutation_denial)
    }

    pub(in crate::domain_computation) fn validate_projection_entry_with_tag(
        &self,
        tag: &BrandTag,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        self.validate_installed_entry_with_tag(tag, runtime, binding, installed)?;
        self.validate_current_authority()
    }
    fn validate_installed_operation(
        &self,
        tag: &BrandTag,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), Denial> {
        let retained = installed.operation(&self.data.operation).ok_or_else(|| {
            Denial::new(
                Kind::Refused(Refusal::StaleInstalledOperation),
                &self.data.operation,
            )
        })?;
        // Borrow the genuine installed handle after releasing the catalog lock.
        self.validate_installed_view(tag, retained.installed_view())
    }

    fn validate_installed_view(
        &self,
        tag: &BrandTag,
        installed: InstalledOperationView<'_>,
    ) -> Result<(), Denial> {
        let refuse = |kind| Denial::new(Kind::Refused(kind), &self.data.operation);
        if self.data.binding_identity != *installed.binding_identity()
            || self.data.operation != installed.operation()
            || self.data.operation_authority_identity_bytes != installed.authority_identity_bytes()
            || self.data.operation_definition_identity != installed.definition_identity_bytes()
            || tag.operation_type != EqualityTypeId::from(installed.operation_type_id())
        {
            return Err(refuse(Refusal::StaleInstalledOperation));
        }
        if tag.input_type != EqualityTypeId::from(installed.input_type_id()) {
            return Err(refuse(Refusal::InputNotAdmitted));
        }
        Ok(())
    }
}

fn mutation_denial(
    denial: Denial,
) -> crate::domain_computation::primary_graph::MutationHandlerExecutionDenial {
    use crate::domain_computation::primary_graph::MutationHandlerExecutionDenial as HandlerDenial;
    match denial.kind() {
        Kind::Refused(Refusal::InputNotAdmitted) => HandlerDenial::InputNotAdmitted,
        _ => HandlerDenial::Projection(denial.into()),
    }
}

#[path = "installed_reentry_court_doors_for_test.rs"]
mod court_doors_for_test;

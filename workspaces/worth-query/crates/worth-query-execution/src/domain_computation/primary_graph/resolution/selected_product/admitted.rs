//! Fresh principal resolution at the already selected Product/native basis.
#[path = "admitted/issued_principal.rs"]
mod issued_principal;
#[path = "admitted/native.rs"]
mod native;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
pub(in crate::domain_computation) use issued_principal::WorthQueryIssuedSelectedPrincipal;
#[path = "admitted/principal_admission_denial.rs"]
mod principal_admission_denial;
use principal_admission_denial::unfunded_principal_admission_denial;

use worth_query_installation::facade::WorthQueryPrincipalBindingValidationAdmissionStop;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalSnapshotProjectionAdmissionStop;

use super::super::*;
use crate::domain_computation::primary_graph::authenticated_principal::PrincipalMintAdmissionStop;
use crate::domain_computation::primary_graph::index_currency::{
    ensure_selected_field_indexes_admitted, SelectedFieldIndexAdmissionStop,
};

impl<'runtime, Schema> super::super::super::WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation) fn resolve_authenticated_principal_issued<
        'selected,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &'selected self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
        validated: Option<&worth_query_installation::facade::WorthQueryValidatedPrincipalBinding>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryIssuedSelectedPrincipal<
            'selected,
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        self.resolve_authenticated_principal_issued_with_native_access(execution, installed_binding, external, request, mode, validated, admission,
            &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_authenticated_principal_issued_with_native_access<
        'selected,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &'selected self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
        validated: Option<&worth_query_installation::facade::WorthQueryValidatedPrincipalBinding>,
        admission: &mut InvalidationEditAdmission,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        WorthQueryIssuedSelectedPrincipal<
            'selected,
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        let principal = self.resolve_authenticated_principal_admitted_with_native_access(
            execution,
            installed_binding,
            external,
            request,
            mode,
            validated,
            admission,
            native,
        )?;
        admission
            .charge_external_work(1)
            .map_err(|stop| principal_admission_denial(stop, installed_binding.binding()))?;
        Ok(WorthQueryIssuedSelectedPrincipal {
            principal,
            selected: self,
            request: request.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_authenticated_principal_admitted<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        self.resolve_authenticated_principal_admitted_with_validation(
            execution,
            installed_binding,
            external,
            scope,
            mode,
            None,
            admission,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_authenticated_principal_admitted_with_validation<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
        validated: Option<&worth_query_installation::facade::WorthQueryValidatedPrincipalBinding>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        self.resolve_authenticated_principal_admitted_with_native_access(
            execution, installed_binding, external, scope, mode, validated, admission,
            &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire(),
        )
    }
}

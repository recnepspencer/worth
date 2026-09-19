use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, ApplicationSchemaMember, WritePosture,
};

use super::{
    installation::{
        WorthQueryConditionalRuntimeInstallationDenial,
        WorthQueryConditionalRuntimeInstallationDenialKind,
    },
    reconstruction_authority::{
        WorthQueryTemporalPrincipalSource, WorthQueryTemporalReconstructionAccess,
    },
};
use crate::domain_computation::primary_graph::application_runtime::installation::ApplicationRuntimePublication;

pub(super) fn validate_reconstruction_access<
    Schema,
    PrincipalBinding,
    PrincipalMapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
    Scope,
    ScopeAspect,
    ScopeField,
    ScopeValue,
    ScopeWrite,
    ScopeUnit,
    PrincipalSource,
    QueryAuthorization,
>(
    publication: &ApplicationRuntimePublication<Schema>,
    access: &WorthQueryTemporalReconstructionAccess<
        Schema,
        PrincipalBinding,
        PrincipalMapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
        QueryAuthorization,
    >,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>
where
    Schema: ApplicationSchema,
    PrincipalIdentityBinding:
        worth_query_installation::facade::ApplicationIdentityScalarValueBinding<
            Value = PrincipalIdentity,
        >,
    ScopeField: worth_query_installation::facade::DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding:
        worth_query_installation::facade::ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
{
    publication
        .runtime
        .installed_packages()
        .validate_principal_binding(&access.principal_binding)
        .map_err(|denial| foreign_binding_denial(denial.binding()))?;
    let field = &access.scope_field;
    let installed = publication
        .installed_schema
        .installed_declaration()
        .members()
        .iter()
        .any(|member| {
            matches!(member,
                ApplicationSchemaMember::Field {
                    entity,
                    aspect,
                    field: member_field,
                    scalar_family,
                    ..
                } if entity == field.entity()
                    && aspect == field.aspect()
                    && member_field == field.field()
                    && *scalar_family == field.scalar_family()
            )
        });
    if installed {
        Ok(())
    } else {
        Err(foreign_binding_denial(field.field()))
    }
}

fn foreign_binding_denial(
    subject: impl Into<String>,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::ForeignBinding,
        subject,
    )
}

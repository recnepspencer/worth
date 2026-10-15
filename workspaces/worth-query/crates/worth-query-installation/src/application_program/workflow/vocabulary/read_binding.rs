//! Typed immutable resolution of one installed disclosure slot; not permission.
use super::*;
use crate::application_capability::WorthQueryInstalledApplicationCapability;
use crate::application_principal_binding::WorthQueryInstalledPrincipalBinding;
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRef,
    application_program::{
        ApplicationWorkflowReadBinding, ApplicationWorkflowReadBindingDeclaration,
        ApplicationWorkflowReadRoute,
    },
    application_schema::ApplicationOperationRef,
};

/// Installed executable type correspondence. Each read still freshly admits its input.
///
/// ```
/// use worth_query_declaration::facade::application_program::{ApplicationWorkflowSpec,ApplicationWorkflowReadBinding,ApplicationWorkflowReadRoute};
/// use worth_query_installation::facade::{ApplicationSchema,WorthQueryInstalledApplicationSchema,WorthQueryInstalledApplicationWorkflowSpec};
/// fn resolve<S,W,B>(schema:&WorthQueryInstalledApplicationSchema<S>,workflow:&WorthQueryInstalledApplicationWorkflowSpec<S,W>)
/// where S:ApplicationSchema,W:ApplicationWorkflowSpec<Schema=S>,B:ApplicationWorkflowReadBinding<S> {
///     let _ = workflow.resolve_read_binding::<B>(schema,ApplicationWorkflowReadRoute::Subject);
/// }
/// ```
pub struct WorthQueryInstalledWorkflowReadBinding<'a, Schema, Spec, B>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    B: ApplicationWorkflowReadBinding<Schema>,
{
    workflow: &'a WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    route: ApplicationWorkflowReadRoute,
    principal: WorthQueryInstalledPrincipalBinding<
        Schema,
        B::PrincipalBinding,
        B::Mapping,
        B::Principal,
        B::PrincipalIdentity,
        B::PrincipalIdentityBinding,
    >,
    capability:
        WorthQueryInstalledApplicationCapability<Schema, B::Capability, B::Operation, B::Input>,
}
impl<Schema, Spec> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// Resolves only the exact concrete binding declared in this installed slot.
    pub fn resolve_read_binding<'a, B: ApplicationWorkflowReadBinding<Schema>>(
        &'a self,
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        route: ApplicationWorkflowReadRoute,
    ) -> Result<
        WorthQueryInstalledWorkflowReadBinding<'a, Schema, Spec, B>,
        WorthQueryApplicationWorkflowInstallationDenial,
    > {
        let invalid = || {
            denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy,
                Spec::IDENTITY.as_str(),
            )
        };
        let declared = self.observation_policy.rule(route).ok_or_else(invalid)?;
        if self.schema_binding() != &schema.binding_identity()
            || declared.binding()
                != &ApplicationWorkflowReadBindingDeclaration::from_binding::<Schema, B>()
        {
            return Err(invalid());
        }
        let principal = schema
            .principal_binding(B::principal_binding())
            .map_err(|_| invalid())?;
        let capability = schema
            .capability(
                ApplicationCapabilityRef::<Schema, B::Capability>::from_declaration(),
                ApplicationOperationRef::<Schema, B::Operation, B::Input>::from_declaration(),
            )
            .map_err(|_| invalid())?;
        Ok(WorthQueryInstalledWorkflowReadBinding {
            workflow: self,
            route,
            principal,
            capability,
        })
    }
}
impl<Schema, Spec, B> WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, B>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    B: ApplicationWorkflowReadBinding<Schema>,
{
    pub fn workflow(&self) -> &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec> {
        self.workflow
    }
    pub fn route(&self) -> ApplicationWorkflowReadRoute {
        self.route
    }
    pub fn principal_binding(
        &self,
    ) -> &WorthQueryInstalledPrincipalBinding<
        Schema,
        B::PrincipalBinding,
        B::Mapping,
        B::Principal,
        B::PrincipalIdentity,
        B::PrincipalIdentityBinding,
    > {
        &self.principal
    }
    pub fn capability(
        &self,
    ) -> &WorthQueryInstalledApplicationCapability<Schema, B::Capability, B::Operation, B::Input>
    {
        &self.capability
    }
}

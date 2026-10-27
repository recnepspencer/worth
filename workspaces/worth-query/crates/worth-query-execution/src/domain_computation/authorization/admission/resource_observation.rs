//! Resource actions observe their own installed requirement on the issued root.
use super::validation::denial;
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryOperationAuthorizationRefusalKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryAuthorizationRequirement, ApplicationQueryBinding,
        ApplicationQueryScopeBinding,
    },
    application_resource::{ApplicationResourceAction, ApplicationResourceBinding},
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationResource;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn observe_resource_authorization<Binding>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: worth_relational::facade::snapshots::SnapshotHandle,
        resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
        action: ApplicationResourceAction,
        access: &WorthQueryApplicationQueryAccessContext<'_, Schema,
            <Binding::Access as ApplicationQueryBinding<Schema>>::Principal,
            <Binding::Access as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            <<Binding::Access as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    where
        Binding: ApplicationResourceBinding<Schema>,
    {
        if resource.binding_identity() != &self.installed_schema().binding_identity() {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::ForeignRuntime,
                ),
                Binding::IDENTITY,
            ));
        }
        let requirement = resource.action_requirement(action);
        if requirement.is_none()
            && !matches!(
                resource
                    .definition()
                    .parts()
                    .authorization
                    .requirement(action),
                ApplicationQueryAuthorizationRequirement::Public
            )
        {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::ForeignRuntime,
                ),
                Binding::IDENTITY,
            ));
        }
        self.observe_authorization_requirements(
            execution,
            WorthQueryConventionalAuthorizationObservation {
                session_identity,
                relational,
                snapshot,
                principal: access.principal(),
                scope_identity: access.scope(),
                binding_identity: resource.binding_identity(),
                requirements: requirement.map(std::slice::from_ref).unwrap_or(&[]),
            },
        )
    }
}

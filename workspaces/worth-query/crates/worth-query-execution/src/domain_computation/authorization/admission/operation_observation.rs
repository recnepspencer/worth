//! Current operation observation and retained-decision construction.

use super::{
    validation::denial, WorthQueryConventionalAuthorizationObservation,
    WorthQueryObservedConventionalOperation,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryPrincipalCurrentnessDependency, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::{
    validate_freshness_at_snapshot, WorthQueryApplicationEntityIdentity,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
use worth_query_installation::facade::{
    ApplicationScalarValueBinding, ApplicationSchema, WorthQueryInstalledApplicationOperation,
    WorthQueryInstalledApplicationOperationAuthorization,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::authorization) fn observe_operation_authorization<
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, WorthQueryOperationAuthorizationDenial>
    {
        self.observe_current_operation(execution, principal, scope_identity, operation, graph_work)?
            .retain_for(operation.installed_view())
    }

    fn observe_current_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<WorthQueryObservedConventionalOperation, WorthQueryOperationAuthorizationDenial>
    {
        let snapshot = graph_work
            .mutation_snapshot()
            .expect("a mutation session owns its admitted snapshot")
            .clone();
        let handle = graph_work
            .mutation_handle()
            .expect("a mutation session owns its graph handle")
            .clone();
        handle.with_runtime(execution, |relational| {
            self.observe_current_operation_borrowed(
                execution,
                principal,
                scope_identity,
                operation.installed_view(),
                graph_work.identity(),
                relational,
                snapshot,
            )
        })
    }

    pub(super) fn observe_current_operation_borrowed<Principal, PrincipalIdentity, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: worth_query_installation::facade::InstalledOperationView<'_>,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<WorthQueryObservedConventionalOperation, WorthQueryOperationAuthorizationDenial>
    {
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ForeignRuntime,
                ),
                operation.operation(),
            )
        })?;

        let principal_layout = graph
            .layout()
            .principal_binding(principal.binding())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::StalePrincipal,
                    ),
                    principal.binding(),
                )
            })?;
        let principal_currentness = WorthQueryPrincipalCurrentnessDependency::capture(
            session_identity,
            principal,
            &principal_layout,
        );
        let entity_resolution = graph.retain_entity_resolution_context();
        let expected_external_identity =
            WorthQueryExternalPrincipalIdentityBinding::encode(principal.external_identity())
                .map_err(|_| {
                    denial(
                        WorthQueryOperationAuthorizationDenialKind::Refused(
                            AuthorizationRefusal::StalePrincipal,
                        ),
                        principal.binding(),
                    )
                })?;
        let decision_facts = {
            validate_freshness_at_snapshot(
                execution,
                relational,
                &snapshot,
                principal,
                &principal_layout,
                &expected_external_identity,
            )
            .map_err(|cause| {
                denial(
                    super::super::denial::principal_currentness(cause),
                    principal.binding(),
                )
            })?;
            entity_resolution
                .at_snapshot(
                    relational,
                    &snapshot,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .and_then(|truth| truth.validate_entity_freshness(execution, scope_identity))
                .map_err(|cause| {
                    match cause.kind() {
                        crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Stopped(cause) => {
                            super::super::denial::relational_read_denial(
                                cause,
                                scope_identity.entity_name(),
                            )
                        }
                        crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(_) => {
                            denial(
                                WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StaleScope),
                                scope_identity.entity_name(),
                            )
                        }
                    }
                })?;
            self.observe_authorization_requirements(
                execution,
                WorthQueryConventionalAuthorizationObservation {
                    session_identity,
                    relational,
                    snapshot,
                    principal,
                    scope_identity,
                    binding_identity: operation.binding_identity(),
                    requirements: operation.contracts().ability_requirements(),
                },
            )
        }?;
        Ok(WorthQueryObservedConventionalOperation {
            session_identity,
            principal_currentness,
            decision_facts,
        })
    }
}

impl WorthQueryObservedConventionalOperation {
    pub(super) fn retain_for(
        self,
        operation: worth_query_installation::facade::InstalledOperationView<'_>,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, WorthQueryOperationAuthorizationDenial>
    {
        let authorization = match operation.contracts().authorization() {
            WorthQueryInstalledApplicationOperationAuthorization::Principal
                if self.decision_facts.is_empty() =>
            {
                WorthQueryRetainedAuthorizationDecisionFacts::principal(self.principal_currentness)
            }
            WorthQueryInstalledApplicationOperationAuthorization::Abilities => {
                WorthQueryRetainedAuthorizationDecisionFacts::abilities(
                    self.principal_currentness,
                    self.decision_facts,
                )
            }
            WorthQueryInstalledApplicationOperationAuthorization::Principal
            | WorthQueryInstalledApplicationOperationAuthorization::Capability => {
                return Err(denial(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::InconsistentDecision,
                    ),
                    operation.operation(),
                ));
            }
        };
        if authorization.belongs_to_session(self.session_identity) {
            Ok(authorization)
        } else {
            Err(denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::InconsistentDecision,
                ),
                operation.operation(),
            ))
        }
    }
}

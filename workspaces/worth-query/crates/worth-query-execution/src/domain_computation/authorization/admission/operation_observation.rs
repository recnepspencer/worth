//! Current operation observation and retained-decision construction.

use super::{
    validation::denial, WorthQueryConventionalAuthorizationObservation,
    WorthQueryObservedConventionalOperation,
};
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
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, WorthQueryOperationAuthorizationDenial>
    {
        self.observe_current_operation(principal, scope_identity, operation, graph_work)?
            .retain_for(operation)
    }

    fn observe_current_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(
        &self,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<WorthQueryObservedConventionalOperation, WorthQueryOperationAuthorizationDenial>
    {
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
                operation.operation(),
            )
        })?;
        let session_identity = graph_work.identity();
        let principal_layout = graph
            .layout()
            .principal_binding(principal.binding())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::StalePrincipal,
                    principal.binding(),
                )
            })?;
        let principal_currentness = WorthQueryPrincipalCurrentnessDependency::capture(
            session_identity,
            principal,
            &principal_layout,
        );
        let snapshot = graph_work
            .mutation_snapshot()
            .expect("a mutation session owns its admitted snapshot")
            .clone();
        let handle = graph_work
            .mutation_handle()
            .expect("a mutation session owns its graph handle")
            .clone();
        let entity_resolution = graph.retain_entity_resolution_context();
        let expected_external_identity =
            WorthQueryExternalPrincipalIdentityBinding::encode(principal.external_identity())
                .map_err(|_| {
                    denial(
                        WorthQueryOperationAuthorizationDenialKind::StalePrincipal,
                        principal.binding(),
                    )
                })?;
        let decision_facts = handle.with_runtime_mut(|relational| {
            validate_freshness_at_snapshot(
                relational,
                &snapshot,
                principal,
                &principal_layout,
                &expected_external_identity,
            )
            .map_err(|_| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::StalePrincipal,
                    principal.binding(),
                )
            })?;
            entity_resolution
                .at_snapshot(
                    relational,
                    &snapshot,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .and_then(|truth| truth.validate_entity_freshness(scope_identity))
                .map_err(|_| {
                    denial(
                        WorthQueryOperationAuthorizationDenialKind::StaleScope,
                        scope_identity.entity_name(),
                    )
                })?;
            self.observe_authorization_requirements(
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
        })?;
        Ok(WorthQueryObservedConventionalOperation {
            session_identity,
            principal_currentness,
            decision_facts,
        })
    }
}

impl WorthQueryObservedConventionalOperation {
    fn retain_for<Schema, Operation, Input>(
        self,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
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
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    operation.operation(),
                ));
            }
        };
        if authorization.belongs_to_session(self.session_identity) {
            Ok(authorization)
        } else {
            Err(denial(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                operation.operation(),
            ))
        }
    }
}

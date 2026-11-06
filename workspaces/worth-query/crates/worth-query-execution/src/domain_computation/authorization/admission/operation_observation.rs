//! Current operation observation and retained-decision construction.

use super::{
    validation::denial, WorthQueryConventionalAuthorizationObservation,
    WorthQueryObservedConventionalOperation,
};
use crate::domain_computation::authorization::admission::requirement_observation::RequirementObservationParentOps;
use crate::domain_computation::authorization::{
    WorthQueryAdmissionPrincipal, WorthQueryAdmissionScope,
};
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
use worth_query_installation::facade::{
    ApplicationScalarValueBinding, ApplicationSchema, WorthQueryInstalledApplicationOperation,
    WorthQueryInstalledApplicationOperationAuthorization,
};
use {
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts,
    crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
};

pub(in crate::domain_computation::authorization) trait OperationObservationScopedOps<Schema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn observe_operation_authorization<Principal, PrincipalIdentity, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &dyn WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &dyn WorthQueryAdmissionScope<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, WorthQueryOperationAuthorizationDenial>
    {
        self.observe_current_operation(execution, principal, scope_identity, operation, graph_work)?
            .retain_for(operation)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > OperationObservationScopedOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

trait OperationObservationOwnOps<Schema>:
    crate::domain_computation::authorization::host::AuthorizationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn observe_current_operation<Principal, PrincipalIdentity, Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        principal: &dyn WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>,
        scope_identity: &dyn WorthQueryAdmissionScope<Schema, Scope>,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<WorthQueryObservedConventionalOperation, WorthQueryOperationAuthorizationDenial>
    {
        let graph = self.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ForeignRuntime,
                ),
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
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::StalePrincipal,
                    ),
                    principal.binding(),
                )
            })?;
        let principal_currentness =
            principal.capture_currentness(session_identity, &principal_layout);
        let snapshot = graph_work
            .mutation_snapshot()
            .expect("a mutation session owns its admitted snapshot")
            .clone();
        let handle = graph_work
            .mutation_handle()
            .expect("a mutation session owns its graph handle")
            .clone();
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
        let decision_facts = handle.with_runtime_mut(execution, |relational| {
            principal
                .validate_freshness_at_snapshot(
                    execution,
                    relational,
                    &snapshot,
                    &principal_layout,
                    &expected_external_identity,
                )
                .map_err(|cause| {
                    denial(super::super::denial::principal_currentness(cause), principal.binding())
                })?;
            scope_identity
                .validate_freshness_at_snapshot(
                    execution,
                    self.authorization_host(),
                    relational,
                    &snapshot,
                )
                .map_err(|kind| {
                    match kind {
                        crate::domain_computation::vocabulary::primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind::Stopped(cause) => {
                            super::super::denial::relational_read_denial(
                                cause,
                                scope_identity.entity_name(),
                            )
                        }
                        crate::domain_computation::vocabulary::primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind::Refused(_) => {
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
        })?;
        Ok(WorthQueryObservedConventionalOperation {
            session_identity,
            principal_currentness,
            decision_facts,
        })
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::authorization::host::AuthorizationHost<Schema> + ?Sized,
    > OperationObservationOwnOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
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

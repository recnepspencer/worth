//! Installed-requirement observation and exact decision-fact minting.

use super::policy_decision::WorthQueryInstalledPolicySubject;
use super::validation::denial;
use super::WorthQueryConventionalAuthorizationDecisionPermit;
use super::WorthQueryConventionalAuthorizationObservation;
use crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn observe_query_authorization_requirement<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: worth_relational::facade::snapshots::SnapshotHandle,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    {
        self.observe_authorization_requirements(
            execution,
            WorthQueryConventionalAuthorizationObservation {
                session_identity,
                relational,
                snapshot,
                principal: access.principal(),
                scope_identity: access.scope(),
                binding_identity: query.binding_identity(),
                requirements: std::slice::from_ref(requirement),
            },
        )
    }

    pub fn observe_authorization_requirements<Principal, PrincipalIdentity, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: WorthQueryConventionalAuthorizationObservation<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    {
        observation.validate_all(execution, self)
    }
}

pub trait ConventionalAuthorizationObservationRuntimeExt<
    Schema,
    Principal,
    PrincipalIdentity,
    Scope,
>: Sized where
    Schema: ApplicationSchema,
{
    fn validate_all(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>;
    fn observe_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial>;
}

impl<Schema, Principal, PrincipalIdentity, Scope>
    ConventionalAuthorizationObservationRuntimeExt<Schema, Principal, PrincipalIdentity, Scope>
    for WorthQueryConventionalAuthorizationObservation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Scope,
    >
where
    Schema: ApplicationSchema,
{
    fn validate_all(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    {
        let mut admitted = Vec::with_capacity(self.requirements.len());
        for requirement in self.requirements {
            admitted.push(self.observe_one(execution, runtime, requirement)?);
        }
        Ok(admitted)
    }

    /// Admission is the installed policy decision plus the one step
    /// eligibility observation never takes: minting the decision fact.
    fn observe_one(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial> {
        let decision = WorthQueryInstalledPolicySubject {
            relational: self.relational,
            snapshot: self.snapshot.clone(),
            principal_entity_id: self.principal.principal_entity_id(),
            scope_identity: self.scope_identity,
            binding_identity: self.binding_identity,
        }
        .decide(execution, runtime, requirement)?;
        if !decision.is_allowed() {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::PermissionDenied,
                ),
                requirement.policy(),
            ));
        }
        let (evidence, bridge) = decision.into_evidence();
        Ok(
            WorthQueryAuthorizationDecisionFact::from_conventional_observation(
                WorthQueryConventionalAuthorizationDecisionPermit::new(),
                self.session_identity,
                evidence,
                bridge,
            ),
        )
    }
}

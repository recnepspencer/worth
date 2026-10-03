//! Installed-requirement observation and exact decision-fact minting.

use super::{
    validation::{denial, validate_decision},
    WorthQueryConventionalAuthorizationDecisionPermit,
    WorthQueryConventionalAuthorizationObservation,
};
use crate::domain_computation::authorization::installed_policy::WorthQueryInstalledAuthorizationPolicy;
use crate::domain_computation::authorization::{
    bridge_observation::lower_bridge_observation, WorthQueryAuthorizationDecisionFact,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::authorization::RelationalAuthorizationObservationPlan;

mod admitted;
pub(in crate::domain_computation) use admitted::AdmittedQueryAuthorizationStop;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation) fn observe_query_authorization_requirement<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
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
        self.observe_authorization_requirements(WorthQueryConventionalAuthorizationObservation {
            session_identity,
            relational,
            snapshot,
            principal: access.principal(),
            scope_identity: access.scope(),
            binding_identity: query.binding_identity(),
            requirements: std::slice::from_ref(requirement),
        })
    }

    pub(super) fn observe_authorization_requirements<Principal, PrincipalIdentity, Scope>(
        &self,
        observation: WorthQueryConventionalAuthorizationObservation<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    {
        observation.validate_all(self)
    }
}

impl<Schema, Principal, PrincipalIdentity, Scope>
    WorthQueryConventionalAuthorizationObservation<'_, Schema, Principal, PrincipalIdentity, Scope>
where
    Schema: ApplicationSchema,
{
    fn validate_all(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<Vec<WorthQueryAuthorizationDecisionFact>, WorthQueryOperationAuthorizationDenial>
    {
        let mut admitted = Vec::with_capacity(self.requirements.len());
        for requirement in self.requirements {
            admitted.push(self.observe_one(runtime, requirement)?);
        }
        Ok(admitted)
    }

    fn observe_one(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<WorthQueryAuthorizationDecisionFact, WorthQueryOperationAuthorizationDenial> {
        self.validate_scope(requirement)?;
        let installed = runtime.authorization.policy(requirement)?;
        self.validate_installation(runtime, installed, requirement)?;
        let evidence = self.observe_relational(installed, requirement)?;
        let dependency_identity = *evidence.observation_identity().bytes();
        let bridge_observation = lower_bridge_observation(
            installed,
            &evidence,
            dependency_identity,
            requirement.policy(),
        )?;
        let bridge = runtime
            .authorization
            .bridge()
            .evaluate(bridge_observation)
            .map_err(|_| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::BridgeEvaluationRejected,
                    requirement.policy(),
                )
            })?;
        validate_decision(
            runtime.authorization.bridge(),
            &evidence,
            &bridge,
            dependency_identity,
            requirement.policy(),
        )?;
        Ok(
            WorthQueryAuthorizationDecisionFact::from_conventional_observation(
                WorthQueryConventionalAuthorizationDecisionPermit::new(),
                self.session_identity,
                evidence,
                bridge,
            ),
        )
    }

    fn validate_scope(
        &self,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if requirement.scope_entity() == self.scope_identity.entity_name() {
            Ok(())
        } else {
            Err(denial(
                WorthQueryOperationAuthorizationDenialKind::ScopeMismatch,
                requirement.scope_entity(),
            ))
        }
    }

    fn validate_installation(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        installed: &WorthQueryInstalledAuthorizationPolicy,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if installed.scope_kind() != self.scope_identity.entity_kind() {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::ScopeMismatch,
                requirement.scope_entity(),
            ));
        }
        let matches = runtime.authorization.bridge().matches_installed_policy(
            installed.correspondence(),
            &crate::domain_computation::authorization::bridge_authorization_binding_identity(
                self.binding_identity,
            ),
            requirement.ability(),
            requirement.scope_entity(),
            requirement.policy(),
            installed.bridge_rule_bindings().iter().map(
                crate::domain_computation::authorization::installed_policy::BridgeRuleBinding::rule,
            ),
        );
        if matches {
            Ok(())
        } else {
            Err(denial(
                WorthQueryOperationAuthorizationDenialKind::PolicyNotInstalled,
                requirement.policy(),
            ))
        }
    }

    fn observe_relational(
        &self,
        installed: &WorthQueryInstalledAuthorizationPolicy,
        requirement: &worth_query_installation::facade::WorthQueryInstalledAbilityRequirement,
    ) -> Result<
        worth_relational::facade::authorization::RelationalAuthorizationObservationEvidence,
        WorthQueryOperationAuthorizationDenial,
    > {
        let plan = RelationalAuthorizationObservationPlan::try_new(
            self.snapshot.clone(),
            self.principal.principal_entity_id(),
            self.scope_identity.entity_id(),
            installed.principal_kind(),
            installed.scope_kind(),
            installed.relational_paths().to_vec(),
            [],
        )
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::InvalidInstalledPolicy,
                requirement.policy(),
            )
        })?;
        self.relational.observe_authorization(plan).map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected,
                requirement.policy(),
            )
        })
    }
}

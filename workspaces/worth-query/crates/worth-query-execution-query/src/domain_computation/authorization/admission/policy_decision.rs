//! One installed ability-policy decision for one principal entity on one scope.
//!
//! This is the single evaluator behind conventional admission and fact-free
//! eligibility observation. Admission consumes an allowed decision and mints
//! its decision fact; eligibility reads the verdict and mints nothing, so the
//! two cannot disagree about who may act.

use super::validation::{denial, validate_decision_consistency};
use crate::domain_computation::authorization::bridge_observation::lower_bridge_observation;
use crate::domain_computation::authorization::installed_policy::{
    BridgeRuleBinding, WorthQueryInstalledAuthorizationPolicy,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    bridge_authorization_binding_identity, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledAbilityRequirement,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationObservationDenial, RelationalAuthorizationObservationEvidence,
    RelationalAuthorizationObservationPlan,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_bridge::facade::BridgeAuthorizationDecisionEvidence;

/// Who is asked about, where, and against which committed snapshot.
pub struct WorthQueryInstalledPolicySubject<'a, Schema, Scope> {
    pub relational: &'a RelationalRuntime,
    pub snapshot: SnapshotHandle,
    pub principal_entity_id: EntityId,
    pub scope_identity: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    pub binding_identity: &'a ApplicationSchemaBindingIdentity,
}

/// A consistent, Bridge-retained decision. It is not authority: only
/// admission may turn an allowed decision into a decision fact.
pub struct WorthQueryInstalledPolicyDecision {
    relational: RelationalAuthorizationObservationEvidence,
    bridge: BridgeAuthorizationDecisionEvidence,
}

impl WorthQueryInstalledPolicyDecision {
    /// Whether every required path holds and no prohibiting path does.
    pub fn is_allowed(&self) -> bool {
        self.bridge.is_allowed()
    }

    pub fn into_evidence(
        self,
    ) -> (
        RelationalAuthorizationObservationEvidence,
        BridgeAuthorizationDecisionEvidence,
    ) {
        (self.relational, self.bridge)
    }
}

impl<Schema, Scope> WorthQueryInstalledPolicySubject<'_, Schema, Scope>
where
    Schema: ApplicationSchema,
{
    /// Evaluates `requirement` for this subject. A refused or stopped
    /// evaluation is a denial; a completed one reports its verdict.
    pub fn decide(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<WorthQueryInstalledPolicyDecision, WorthQueryOperationAuthorizationDenial> {
        self.validate_scope(requirement)?;
        let installed = runtime.runtime.authorization.policy(requirement)?;
        self.validate_installation(runtime, installed, requirement)?;
        let relational = self.observe_relational(execution, installed, requirement)?;
        let dependency_identity = *relational.observation_identity().bytes();
        let observation = lower_bridge_observation(installed, &relational, requirement.policy())?;
        let bridge_runtime = runtime.runtime.authorization.bridge();
        let bridge = bridge_runtime.evaluate(observation).map_err(|_| {
            refused(
                AuthorizationRefusal::BridgeEvaluationRejected,
                requirement.policy(),
            )
        })?;
        validate_decision_consistency(
            bridge_runtime,
            &relational,
            &bridge,
            dependency_identity,
            requirement.policy(),
        )?;
        Ok(WorthQueryInstalledPolicyDecision { relational, bridge })
    }

    fn validate_scope(
        &self,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if requirement.scope_entity() == self.scope_identity.entity_name() {
            Ok(())
        } else {
            Err(refused(
                AuthorizationRefusal::ScopeMismatch,
                requirement.scope_entity(),
            ))
        }
    }

    fn validate_installation(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        installed: &WorthQueryInstalledAuthorizationPolicy,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if installed.scope_kind() != self.scope_identity.entity_kind() {
            return Err(refused(
                AuthorizationRefusal::ScopeMismatch,
                requirement.scope_entity(),
            ));
        }
        let matches = runtime
            .runtime
            .authorization
            .bridge()
            .matches_installed_policy(
                installed.correspondence(),
                &bridge_authorization_binding_identity(self.binding_identity),
                requirement.ability(),
                requirement.scope_entity(),
                requirement.policy(),
                installed
                    .bridge_rule_bindings()
                    .iter()
                    .map(BridgeRuleBinding::rule),
            );
        if matches {
            Ok(())
        } else {
            Err(refused(
                AuthorizationRefusal::PolicyNotInstalled,
                requirement.policy(),
            ))
        }
    }

    fn observe_relational(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &WorthQueryInstalledAuthorizationPolicy,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<RelationalAuthorizationObservationEvidence, WorthQueryOperationAuthorizationDenial>
    {
        let plan = RelationalAuthorizationObservationPlan::try_new(
            self.snapshot.clone(),
            self.principal_entity_id,
            self.scope_identity.entity_id(),
            installed.principal_kind(),
            installed.scope_kind(),
            installed.relational_paths().to_vec(),
        )
        .map_err(|_| {
            refused(
                AuthorizationRefusal::InvalidInstalledPolicy,
                requirement.policy(),
            )
        })?;
        self.relational
            .observe_authorization(plan, execution)
            .map_err(|stop| match stop {
                RelationalAuthorizationObservationDenial::Execution(cause) => {
                    crate::domain_computation::authorization::denial::relational_read_denial(
                        cause,
                        requirement.policy(),
                    )
                }
                _ => refused(
                    AuthorizationRefusal::RelationalObservationRejected,
                    requirement.policy(),
                ),
            })
    }
}

fn refused(refusal: AuthorizationRefusal, subject: &str) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(refusal),
        subject,
    )
}

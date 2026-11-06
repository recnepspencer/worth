//! The attempt's request to register one application attempt in the provider store.
//! Only the provider's registration operation takes it apart; only the effect owner,
//! holding its registration seal, can build one.

/// Everything the provider store needs to bind one attempt's records into the
/// operation transaction.
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptRegistration<'a>
{
    pub(super) workflow_origin: Option<crate::domain_computation::primary_graph::application_attempt::workflow::operation_output::WorkflowOperationOrigin>,
    pub(super) required_output_demand: Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
    pub(super) effect_owner: crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryProviderEffectRegistrationSeal,
    pub(super) affinity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptAffinity,
    pub(super) decision_facts: crate::domain_computation::authorization::WorthQueryProviderDecisionFactBinding,
    pub(super) effects: crate::domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::WorthQueryRegisteredProviderEffects,
    pub(super) idempotency: crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
    pub(super) original_authority: std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding>,
    pub(super) outcome_identity: crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome_identity::WorthQueryApplicationCommitOutcomeIdentity,
    pub(super) retained_authorization_fact_count: usize,
    pub(super) external_effect: &'a worth_query_installation::facade::InstalledExternalEffectContract,
    pub(super) preimage_demand: Option<&'a worth_query_installation::facade::InstalledPreImageDemand>,
    pub(super) aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    pub(super) conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    pub(super) retain_output_demand_observation: bool,
    pub(super) retain_client_observation: bool,
    pub(super) producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerInvariantRequirement],
    pub(super) source_fact_rebase: crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase,
    pub(super) consumed_outputs: Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
}

/// Which successor observations the attempt's World publication retains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRetainedSuccessorObservations {
    pub(in crate::domain_computation::primary_graph) output_demand: bool,
    pub(in crate::domain_computation::primary_graph) client: bool,
}

impl<'a> WorthQueryApplicationAttemptRegistration<'a> {
    /// The effect owner's seal proves it consumed a completed provider attempt
    /// before handing its associations to registration.
    pub(in crate::domain_computation::primary_graph) fn new(
        workflow_origin: Option<crate::domain_computation::primary_graph::application_attempt::workflow::operation_output::WorkflowOperationOrigin>,
        required_output_demand: Option<
            crate::domain_computation::primary_graph::RequiredOutputDemandContext,
        >,
        effect_owner: crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryProviderEffectRegistrationSeal,
        affinity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptAffinity,
        decision_facts: crate::domain_computation::authorization::WorthQueryProviderDecisionFactBinding,
        effects: crate::domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::WorthQueryRegisteredProviderEffects,
        idempotency: crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
        original_authority: std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding,
        >,
        outcome_identity: crate::domain_computation::primary_graph::vocabulary::application_attempt::commit_outcome_identity::WorthQueryApplicationCommitOutcomeIdentity,
        retained_authorization_fact_count: usize,
        external_effect: &'a worth_query_installation::facade::InstalledExternalEffectContract,
        preimage_demand: Option<&'a worth_query_installation::facade::InstalledPreImageDemand>,
        aftermath_causality: Option<
            crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
        conditional_definition: Option<
            crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition,
        >,
        retained_observations: WorthQueryRetainedSuccessorObservations,
        producer_required_invariants:
            &'static [crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerInvariantRequirement],
        source_fact_rebase: crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase,
        consumed_outputs: Vec<
            crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence,
        >,
    ) -> Self {
        Self {
            workflow_origin,
            required_output_demand,
            effect_owner,
            affinity,
            decision_facts,
            effects,
            idempotency,
            original_authority,
            outcome_identity,
            retained_authorization_fact_count,
            external_effect,
            preimage_demand,
            aftermath_causality,
            conditional_definition,
            retain_output_demand_observation: retained_observations.output_demand,
            retain_client_observation: retained_observations.client,
            producer_required_invariants,
            source_fact_rebase,
            consumed_outputs,
        }
    }
}

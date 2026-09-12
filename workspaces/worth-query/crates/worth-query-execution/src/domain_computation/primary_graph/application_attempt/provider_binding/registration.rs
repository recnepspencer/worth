use super::super::provider_execution::{
    progression_denied, WorthQueryProviderAttemptRegistrationContext,
    WorthQueryProviderProgressionOutcome, WorthQueryRegisteredProviderAttempt,
};
use super::super::WorthQueryApplicationCommitDenialStage as DenialStage;
use super::WorthQueryPreparedApplicationProviderAttempt;

mod provider_registration;
pub(in crate::domain_computation::primary_graph) use provider_registration::WorthQueryPrimaryGraphApplicationAttempt;

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptRegistration<'a>
{
    effect_owner: WorthQueryProviderEffectRegistrationSeal,
    affinity: super::super::provider_execution::WorthQueryApplicationAttemptAffinity,
    decision_facts: crate::domain_computation::authorization::WorthQueryProviderDecisionFactBinding,
    effects: super::effect_accumulator::WorthQueryRegisteredProviderEffects,
    idempotency: super::super::WorthQueryApplicationIdempotencyBinding,
    retained_authorization_fact_count: usize,
    external_effect: &'a worth_query_installation::facade::InstalledExternalEffectContract,
    preimage_demand: Option<&'a worth_query_installation::facade::InstalledPreImageDemand>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    validator_work_admission:
        super::super::effect_program::WorthQueryCandidateValidatorWorkAdmission,
}

/// Proves that the effect owner consumed a completed provider attempt before
/// handing its associations to Primary Graph registration.
pub(in crate::domain_computation::primary_graph) struct WorthQueryProviderEffectRegistrationSeal {
    _owner_mint: (),
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryRegisteredProviderAttemptSeal
{
    _owner_mint: (),
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryProviderRegistrationInspectionPermit(
    (),
);

impl WorthQueryProviderRegistrationInspectionPermit {
    fn mint() -> Self {
        Self(())
    }
}

impl WorthQueryProviderEffectRegistrationSeal {
    fn mint() -> Self {
        Self { _owner_mint: () }
    }
}

pub(super) fn register_provider_attempt<'run, Schema, Operation, Input, Scope>(
    prepared: WorthQueryPreparedApplicationProviderAttempt,
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    authorization: crate::domain_computation::authorization::WorthQueryProviderAuthorizationDecisionFacts,
    attempt_basis: super::super::provider_execution::WorthQueryApplicationAttemptBasis,
    context: WorthQueryProviderAttemptRegistrationContext<'_, Schema, Operation, Input, Scope>,
) -> Result<WorthQueryRegisteredProviderAttempt<'run>, WorthQueryProviderProgressionOutcome> {
    let inspection = WorthQueryProviderRegistrationInspectionPermit::mint();
    let WorthQueryPreparedApplicationProviderAttempt {
        installed_read_scopes,
        facts,
        effects,
        preimage_demand,
        conditional_definition,
        validator_work_admission,
    } = prepared;
    let affinity = match staged.bind_application_attempt(attempt_basis) {
        Ok(affinity) => affinity,
        Err(()) => return abort_registration(staged, DenialStage::ProviderPlan),
    };
    let decision_facts = match authorization.bind_application_facts(installed_read_scopes, facts) {
        Ok(bound) => bound,
        Err(()) => {
            return abort_registration(staged, DenialStage::DecisionReadSet);
        }
    };
    let expected_steps = effects.expected_steps();
    let dispatch_outbox = context.provider(&inspection).register_application_attempt(
        WorthQueryApplicationAttemptRegistration {
            effect_owner: WorthQueryProviderEffectRegistrationSeal::mint(),
            affinity,
            decision_facts,
            effects,
            idempotency: context.idempotency(&inspection),
            retained_authorization_fact_count: context
                .admission(&inspection)
                .graph_work_decision_fact_count(),
            external_effect: context
                .admission(&inspection)
                .allowed_graph_contract()
                .external_effect(),
            preimage_demand: preimage_demand.as_ref(),
            aftermath_causality: context.aftermath_causality(&inspection).cloned(),
            conditional_definition,
            validator_work_admission,
        },
    );
    match dispatch_outbox {
        Ok(completion) => Ok(completion.finish(
            WorthQueryRegisteredProviderAttemptSeal { _owner_mint: () },
            staged,
            expected_steps,
        )),
        Err(_) => abort_registration(staged, DenialStage::ProviderPlan),
    }
}

fn abort_registration<'run, T>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    stage: DenialStage,
) -> Result<T, WorthQueryProviderProgressionOutcome> {
    let _ = staged.abort();
    Err(progression_denied(stage))
}

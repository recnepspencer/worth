use super::super::provider_execution::{
    progression_denied, WorthQueryProviderAttemptRegistrationContext,
    WorthQueryProviderProgressionOutcome, WorthQueryRegisteredProviderAttempt,
};
use super::super::WorthQueryApplicationCommitDenialStage as DenialStage;
use super::WorthQueryPreparedApplicationProviderAttempt;

mod provider_registration;
use provider_registration::ApplicationAttemptRegistrationStop as RegistrationStop;
pub(in crate::domain_computation::primary_graph) use provider_registration::WorthQueryPrimaryGraphApplicationAttempt;

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptRegistration<'a>
{
    required_output_demand: Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
    effect_owner: WorthQueryProviderEffectRegistrationSeal,
    affinity: super::super::provider_execution::WorthQueryApplicationAttemptAffinity,
    decision_facts: crate::domain_computation::authorization::WorthQueryProviderDecisionFactBinding,
    effects: super::effect_accumulator::WorthQueryRegisteredProviderEffects,
    idempotency: super::super::WorthQueryApplicationIdempotencyBinding,
    outcome_identity: super::super::WorthQueryApplicationCommitOutcomeIdentity,
    retained_authorization_fact_count: usize,
    external_effect: &'a worth_query_installation::facade::InstalledExternalEffectContract,
    preimage_demand: Option<&'a worth_query_installation::facade::InstalledPreImageDemand>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    indexed_rebase_work_budget: usize,
    retain_output_demand_observation: bool,
    retain_client_observation: bool,
    producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    source_fact_rebase: crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase,
    consumed_outputs: Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
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
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<WorthQueryRegisteredProviderAttempt<'run>, WorthQueryProviderProgressionOutcome> {
    let inspection = WorthQueryProviderRegistrationInspectionPermit::mint();
    let WorthQueryPreparedApplicationProviderAttempt {
        required_output_demand,
        installed_read_scopes,
        facts,
        effects,
        preimage_demand,
        conditional_definition,
        retain_output_demand_observation,
        retain_client_observation,
        producer_required_invariants,
        output_currentness_facts,
        consumed_outputs,
    } = prepared;
    let affinity = match staged.bind_application_attempt(attempt_basis) {
        Ok(affinity) => affinity,
        Err(()) => return abort_registration(staged, DenialStage::ProviderPlan),
    };
    let decision_facts = match authorization.bind_application_facts(installed_read_scopes, facts) {
        Ok(bound) => bound,
        Err(detail) => {
            let _ = staged.abort();
            return Err(
                super::super::provider_execution::WorthQueryProviderProgressionOutcome::Denied(
                    super::super::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
                        DenialStage::DecisionReadSet,
                        detail,
                    ),
                ),
            );
        }
    };
    let (source_facts, moved_by_own_effect) = match &output_currentness_facts {
        Some(facts) => (facts.facts().to_vec(), facts.moved_by_own_effect()),
        None => (
            decision_facts
                .facts()
                .values()
                .filter_map(|fact| fact.observed_source_fact().cloned())
                .collect(),
            std::sync::Arc::from([]),
        ),
    };
    // Preserve producer source order or authorization-merged decision fact order.
    // Temporary Vec/other nested heaps remain separately uncharged.
    let allocation_control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(context.admission(&inspection).publication_request(), allocation_policy);
    let source_fact_rebase = match crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase::admit(
        source_facts,
        moved_by_own_effect,
        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(allocation_control.policy(), Some(context.admission(&inspection).publication_request())),
    ) {
        Ok(prepared) => prepared,
        Err(denial) => {
            let _ = staged.abort();
            let denial = super::super::WorthQueryApplicationCommitDenial::source_rebase_denied(denial);
            use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Stop;
            return Err(match denial.source_rebase_interruption() {
                Some(Stop::Cancelled) => WorthQueryProviderProgressionOutcome::Cancelled,
                Some(Stop::DeadlineExceeded) => WorthQueryProviderProgressionOutcome::TimedOut,
                None => WorthQueryProviderProgressionOutcome::Denied(denial),
            });
        }
    };
    let expected_steps = effects.shared_expected_steps();
    let dispatch_outbox = context.provider(&inspection).register_application_attempt(
        WorthQueryApplicationAttemptRegistration {
            required_output_demand,
            effect_owner: WorthQueryProviderEffectRegistrationSeal::mint(),
            affinity,
            decision_facts,
            effects,
            idempotency: context.idempotency(&inspection),
            outcome_identity: context.outcome_identity(&inspection),
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
            indexed_rebase_work_budget: context
                .admission(&inspection)
                .allowed_graph_contract()
                .projection_work_budget(),
            retain_output_demand_observation,
            retain_client_observation,
            producer_required_invariants,
            source_fact_rebase,
            consumed_outputs,
        },
    );
    match dispatch_outbox {
        Ok(completion) => Ok(completion.finish(
            WorthQueryRegisteredProviderAttemptSeal { _owner_mint: () },
            staged,
            expected_steps,
        )),
        Err(RegistrationStop::Interrupted(interruption)) => {
            use worth_relational::facade::mvcc::RelationalOperationInterruption as Interruption;
            let _ = staged.abort();
            Err(match interruption {
                Interruption::Cancelled => WorthQueryProviderProgressionOutcome::Cancelled,
                Interruption::TimedOut => WorthQueryProviderProgressionOutcome::TimedOut,
            })
        }
        Err(RegistrationStop::Rejected(_)) => abort_registration(staged, DenialStage::ProviderPlan),
    }
}

fn abort_registration<'run, T>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    stage: DenialStage,
) -> Result<T, WorthQueryProviderProgressionOutcome> {
    let _ = staged.abort();
    Err(progression_denied(stage))
}

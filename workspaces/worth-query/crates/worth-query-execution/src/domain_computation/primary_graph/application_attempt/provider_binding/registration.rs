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
    workflow_origin: Option<crate::domain_computation::primary_graph::workflow::operation_output::WorkflowOperationOrigin>,
    required_output_demand: Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
    effect_owner: WorthQueryProviderEffectRegistrationSeal,
    affinity: super::super::provider_execution::WorthQueryApplicationAttemptAffinity,
    decision_facts: crate::domain_computation::authorization::WorthQueryProviderDecisionFactBinding,
    effects: super::effect_accumulator::WorthQueryRegisteredProviderEffects,
    idempotency: super::super::WorthQueryApplicationIdempotencyBinding,
    original_authority: std::sync::Arc<super::super::WorthQueryApplicationCommitAuthorityBinding>,
    outcome_identity: super::super::WorthQueryApplicationCommitOutcomeIdentity,
    retained_authorization_fact_count: usize,
    external_effect: &'a worth_query_installation::facade::InstalledExternalEffectContract,
    preimage_demand: Option<&'a worth_query_installation::facade::InstalledPreImageDemand>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
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
        workflow_origin,
        local_action_settlement,
        required_output_demand,
        installed_read_scopes,
        facts,
        application_projection,
        original_read,
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
        Err(()) => {
            return abort_registration_with_detail(
                staged,
                "application attempt does not match the live provider session".into(),
            )
        }
    };
    if original_read
        .as_ref()
        .is_some_and(|read| !affinity.owns_original_read(read))
    {
        return abort_registration(staged, DenialStage::DecisionReadSet);
    }
    let allocation_control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(context.admission(&inspection).publication_request(), allocation_policy);
    let control = crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(allocation_control.policy(), Some(context.admission(&inspection).publication_request()));
    // The completed handler payload precedes fresh authorization's overlay.
    // Correctable ordinary sources retain all semantic reads, including the
    // projected prefix, rather than only output-source observations.
    let correction_facts = context
        .admission(&inspection)
        .allowed_graph_contract()
        .aftermath()
        .filter(|contract| contract.correction_disclosure().is_some())
        .map(|_| facts.clone());
    let application = crate::domain_computation::primary_graph::CompletedApplicationFacts::new(
        installed_read_scopes,
        facts,
    )
    .with_completed_projection(application_projection, original_read);
    let application = match application.admit_binding(control, authorization.fact_count()) {
        Ok(application) => application,
        Err(failure) => {
            let _ = staged.abort();
            return Err(super::super::provider_compare_denial::read_set_failure(
                failure,
            ));
        }
    };
    let decision_facts = match authorization.bind_application_facts(application) {
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
    let source_fact_rebase = match match (&output_currentness_facts, correction_facts) {
        (Some(facts), _) => crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase::admit_output(
            facts.facts().clone(), facts.moved_by_own_effect(), facts.rebase_carry(), control,
        ),
        (None, Some(facts)) => {
            let semantic = match &workflow_origin {
                Some(origin) => origin.semantic_facts(&facts, control),
                None => Ok(facts),
            };
            semantic.map_err(Into::into).and_then(|facts|
                crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase::admit_correction(facts, control))
        },
        (None, None) => crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase::admit(
            decision_facts.facts().explicit_values().filter_map(|fact| fact.observed_source_fact().cloned()).collect(),
            std::sync::Arc::from([]), control,
        ),
    } {
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
    if let Some(source) = context
        .aftermath_causality(&inspection)
        .map(|causality| causality.prepared_correction())
        && let Err(denial) = source.validate_authored_candidate(
            effects.native_publication_roles(),
            effects.batch(),
            context.admission(&inspection).publication_request(),
        )
    {
        let _ = staged.abort();
        use crate::domain_computation::primary_graph::product_operation::correction_source::AuthoredCorrectionCandidateDenial;
        use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Stop;
        return Err(match denial {
            AuthoredCorrectionCandidateDenial::Identity(detail) => {
                WorthQueryProviderProgressionOutcome::Denied(
                    super::super::WorthQueryApplicationCommitDenial::correction_identity_mismatch(
                        detail,
                    ),
                )
            }
            AuthoredCorrectionCandidateDenial::Interrupted(Stop::Cancelled) => {
                WorthQueryProviderProgressionOutcome::Cancelled
            }
            AuthoredCorrectionCandidateDenial::Interrupted(Stop::DeadlineExceeded) => {
                WorthQueryProviderProgressionOutcome::TimedOut
            }
        });
    }
    if let Some(source) = context
        .aftermath_causality(&inspection)
        .map(|causality| causality.prepared_correction())
        && let Err(denial) = source.validate_workflow_candidate(effects.batch())
    {
        let _ = staged.abort();
        return Err(WorthQueryProviderProgressionOutcome::Denied(
            super::super::WorthQueryApplicationCommitDenial::workflow_settlement_denied(&denial),
        ));
    }
    let no_external_effect =
        worth_query_installation::facade::InstalledExternalEffectContract::None;
    let external_effect = match effects.selected_external_contract(
        context
            .admission(&inspection)
            .allowed_graph_contract()
            .external_effect(),
        local_action_settlement.as_ref(),
    ) {
        Ok(contract) => contract.unwrap_or(&no_external_effect),
        Err(detail) => return abort_registration_with_detail(staged, detail.into()),
    };
    let dispatch_outbox = context.provider(&inspection).register_application_attempt(
        WorthQueryApplicationAttemptRegistration {
            workflow_origin,
            required_output_demand,
            effect_owner: WorthQueryProviderEffectRegistrationSeal::mint(),
            affinity,
            decision_facts,
            effects,
            idempotency: context.idempotency(&inspection),
            original_authority: context.original_authority(&inspection),
            outcome_identity: context.outcome_identity(&inspection),
            retained_authorization_fact_count: context
                .admission(&inspection)
                .graph_work_decision_fact_count(),
            external_effect,
            preimage_demand: preimage_demand.as_ref(),
            aftermath_causality: context.aftermath_causality(&inspection).cloned(),
            conditional_definition,
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
        Err(RegistrationStop::Rejected(cause)) => abort_registration_with_detail(
            staged,
            format!("application attempt registration: {cause:?}"),
        ),
    }
}

fn abort_registration_with_detail<T>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'_>,
    detail: String,
) -> Result<T, WorthQueryProviderProgressionOutcome> {
    let _ = staged.abort();
    Err(WorthQueryProviderProgressionOutcome::Denied(
        super::super::WorthQueryApplicationCommitDenial::provider_rejected_with_detail(
            DenialStage::ProviderPlan,
            detail,
        ),
    ))
}

fn abort_registration<T>(
    staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'_>,
    stage: DenialStage,
) -> Result<T, WorthQueryProviderProgressionOutcome> {
    let _ = staged.abort();
    Err(progression_denied(stage))
}

use worth_query_installation::facade::ApplicationSchema;

use super::super::super::provider_binding::{installed_preimage_demand, prepare_provider_attempt};
use super::super::super::{
    provider_recomparison::recover_equivalent_commit_evidence,
    WorthQueryApplicationCommitAuthorityBinding, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialStage as DenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryCommittedReceiptProjection,
};
use super::super::aftermath_resolution::resolve_exact_committed_aftermath;
use super::super::elevation_currentness::WorthQueryElevationCommitCurrentness;
use super::super::outcome::commit_outcome_from_authorization_denial;
use super::super::provider_denial::{denied, denied_with_detail};
use crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality;
use crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization;
use crate::domain_computation::primary_graph::application_attempt::{
    effect_program::WorthQueryApplicationRealizedEffect,
    provider_binding::WorthQueryPreparedApplicationProviderAttempt,
    snapshot_lease::WorthQueryApplicationSnapshotLease, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) mod running;

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) struct WorthQueryPreparedApplicationCommit<
    Schema,
    Operation,
    Input,
    Scope,
> {
    admission: crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    lease: WorthQueryApplicationSnapshotLease,
    provider_attempt: WorthQueryPreparedApplicationProviderAttempt,
    platform_mutation: bool,
    authorization: WorthQueryProviderCommitAuthorization,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryEarlyEquivalentCommitReceiptPermit
{
    _owner_mint: (),
}

impl WorthQueryEarlyEquivalentCommitReceiptPermit {
    fn mint() -> Self {
        Self { _owner_mint: () }
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) enum WorthQueryApplicationCommitPreparation<
    Schema,
    Operation,
    Input,
    Scope,
> {
    Ready(WorthQueryPreparedApplicationCommit<Schema, Operation, Input, Scope>),
    Terminal(WorthQueryApplicationCommitOutcome),
}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) struct WorthQueryApplicationCommitPreparationRequest<
    Schema,
    Operation,
    Input,
    Scope,
> {
    program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
    aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitPreparationRequest<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn new(
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> Self {
        Self {
            program,
            idempotency,
            elevation_currentness,
            aftermath_causality,
        }
    }
}

struct WorthQueryProviderAttemptPreparation {
    installed_read_scopes: Vec<worth_query_installation::facade::WorthQueryOperationGraphReadScope>,
    facts: Vec<WorthQueryApplicationObservedFact>,
    effects: Vec<WorthQueryApplicationRealizedEffect>,
    emission_retained_bytes: u64,
    emission_retained_bytes_ceiling: u64,
    preimage_demand: Option<worth_query_installation::facade::InstalledPreImageDemand>,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    platform_mutation: bool,
    validator_work_admission: crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryCandidateValidatorWorkAdmission,
    output_correspondence: super::super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    retain_output_demand_observation: bool,
    retain_client_observation: bool,
    producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    output_currentness_facts: Option<std::sync::Arc<[WorthQueryApplicationObservedFact]>>,
}

struct WorthQueryCurrentApplicationCommit<Schema, Operation, Input, Scope> {
    admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: WorthQueryApplicationSnapshotLease,
    provider: WorthQueryProviderAttemptPreparation,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn prepare_application_commit<
    Schema,
    Operation,
    Input,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    request: WorthQueryApplicationCommitPreparationRequest<Schema, Operation, Input, Scope>,
) -> WorthQueryApplicationCommitPreparation<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Input: Clone + Send + Sync + 'static,
{
    let WorthQueryApplicationCommitPreparationRequest {
        program,
        idempotency,
        elevation_currentness,
        aftermath_causality,
    } = request;
    let WorthQueryApplicationEffectProgram {
        read_set,
        effects,
        emission_retained_bytes,
        emission_retained_bytes_ceiling,
        conditional_definition,
        platform_mutation,
        validator_work_admission,
        output_correspondence,
        retain_output_demand_observation,
        retain_client_observation,
        producer_required_invariants,
        output_currentness_facts,
    } = program;
    let mut admission = read_set.admission;
    let preimage_demand = installed_preimage_demand(admission.allowed_graph_contract().aftermath());
    let idempotency =
        bind_commit_idempotency(&admission, conditional_definition.as_ref(), idempotency);
    if let Err(outcome) = validate_operation_currentness(&admission) {
        return terminal(outcome);
    }
    if let Some(outcome) = resolve_retained_idempotency(
        application,
        &mut admission,
        idempotency,
        aftermath_causality.as_ref(),
    ) {
        return terminal(outcome);
    }
    if let Err(outcome) = validate_elevation_currentness(application, elevation_currentness) {
        return terminal(outcome);
    }
    prepare_authorized_application_commit(
        application,
        WorthQueryCurrentApplicationCommit {
            admission,
            lease: read_set.lease,
            provider: WorthQueryProviderAttemptPreparation {
                installed_read_scopes: read_set.installed_read_scopes,
                facts: read_set.facts,
                effects,
                emission_retained_bytes,
                emission_retained_bytes_ceiling,
                preimage_demand,
                conditional_definition,
                platform_mutation,
                validator_work_admission,
                output_correspondence,
                retain_output_demand_observation,
                retain_client_observation,
                producer_required_invariants,
                output_currentness_facts,
            },
            idempotency,
            aftermath_causality,
        },
    )
}

fn prepare_authorized_application_commit<Schema, Operation, Input, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    current: WorthQueryCurrentApplicationCommit<Schema, Operation, Input, Scope>,
) -> WorthQueryApplicationCommitPreparation<Schema, Operation, Input, Scope> {
    let WorthQueryCurrentApplicationCommit {
        mut admission,
        lease,
        provider,
        idempotency,
        aftermath_causality,
    } = current;
    let authorization = match take_commit_authorization(application, &mut admission) {
        Ok(authorization) => authorization,
        Err(outcome) => return terminal(outcome),
    };
    let Some(mutation_partition) = application.issue_application_mutation_partition() else {
        return terminal(denied(DenialStage::ProposalBinding));
    };
    let platform_mutation = provider.platform_mutation;
    let provider_attempt = match prepare_application_provider_attempt(provider, mutation_partition)
    {
        Ok(prepared) => prepared,
        Err(_) => return terminal(denied(DenialStage::ProposalBinding)),
    };
    WorthQueryApplicationCommitPreparation::Ready(WorthQueryPreparedApplicationCommit {
        admission,
        lease,
        provider_attempt,
        platform_mutation,
        authorization,
        idempotency,
        aftermath_causality,
    })
}

fn prepare_application_provider_attempt(
    preparation: WorthQueryProviderAttemptPreparation,
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> Result<WorthQueryPreparedApplicationProviderAttempt, ()> {
    prepare_provider_attempt(
        mutation_partition,
        preparation.installed_read_scopes,
        preparation.facts,
        preparation.effects,
        preparation.emission_retained_bytes,
        preparation.emission_retained_bytes_ceiling,
        preparation.preimage_demand,
        preparation.conditional_definition,
        preparation.validator_work_admission,
        preparation.output_correspondence,
        preparation.retain_output_demand_observation,
        preparation.retain_client_observation,
        preparation.producer_required_invariants,
        preparation.output_currentness_facts,
    )
    .map_err(|_| ())
}

fn validate_operation_currentness<Schema, Operation, Input, Scope>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    admission.validate_current_authority().map_err(|denial| {
        commit_outcome_from_authorization_denial(denial, DenialStage::DecisionReadSet)
    })
}

fn validate_elevation_currentness<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    if elevation_currentness
        .as_ref()
        .is_some_and(|currentness| !currentness.remains_current(&application.authorization_clock))
    {
        Err(denied(DenialStage::DecisionReadSet))
    } else {
        Ok(())
    }
}

fn take_commit_authorization<Schema, Operation, Input, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
) -> Result<WorthQueryProviderCommitAuthorization, WorthQueryApplicationCommitOutcome> {
    admission
        .take_authorization_dependencies(application.authorization.bridge())
        .map_err(|denial| denied_with_detail(DenialStage::DecisionReadSet, denial.to_string()))
}

fn bind_commit_idempotency<Schema, Operation, Input, Scope>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    conditional_definition: Option<
        &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition,
    >,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationIdempotencyBinding {
    idempotency
        .bind_operation(admission.operation_authority_identity_bytes())
        .bind_operation_scope(admission.operation_scope_binding())
        .bind_preconditions(admission.mutation_preconditions().identity())
        .bind_governed_input(admission.governed_input_identity())
        .bind_governed_proposal(admission.governed_proposal_identity())
        .bind_conditional_definition(
            conditional_definition.map(
                crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition::identity,
            ),
        )
}

mod retained_idempotency;
use retained_idempotency::resolve_retained_idempotency;

const fn terminal<Schema, Operation, Input, Scope>(
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitPreparation<Schema, Operation, Input, Scope> {
    WorthQueryApplicationCommitPreparation::Terminal(outcome)
}

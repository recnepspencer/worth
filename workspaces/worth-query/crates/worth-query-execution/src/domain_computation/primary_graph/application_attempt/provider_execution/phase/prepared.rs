use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_execution::ExecutionArray;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::provider_binding::installed_preimage_demand;
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

mod currentness;
mod local_workflow_settlement;
use currentness::{
    readmit_current_basis, select_current_product, validate_elevation_currentness,
    validate_operation_currentness, validate_workflow_deadline,
};
pub(super) mod running;
pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) use local_workflow_settlement::LocalWorkflowSettlementPublication;

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
    outcome_identity: super::super::super::WorthQueryApplicationCommitOutcomeIdentity,
    workflow_settlement_publication: Option<LocalWorkflowSettlementPublication>,
    effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture,
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
    required_output_demand: Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
    installed_read_scopes: ExecutionArray<worth_query_installation::facade::WorthQueryOperationGraphReadScope>,
    facts: ExecutionArray<WorthQueryApplicationObservedFact>,
    consumed_outputs: Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    effects: Vec<WorthQueryApplicationRealizedEffect>,
    application_effect_count: usize,
    emission_retained_bytes: u64,
    emission_retained_bytes_ceiling: u64,
    preimage_demand: Option<worth_query_installation::facade::InstalledPreImageDemand>,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture,
    output_correspondence: super::super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    retain_output_demand_observation: bool,
    retain_client_observation: bool,
    producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    output_currentness_facts: Option<super::super::super::OutputCurrentnessFacts>,
    workflow_settlement: Option<
        crate::domain_computation::primary_graph::application_attempt::read_set::WorkflowOperationBindingProof,
    >,
}

struct WorthQueryCurrentApplicationCommit<Schema, Operation, Input, Scope> {
    admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: WorthQueryApplicationSnapshotLease,
    provider: WorthQueryProviderAttemptPreparation,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    workflow_approval_authority:
        Option<crate::domain_computation::authorization::WorthQueryWorkflowApprovalAuthorityBasis>,
}

mod commit_preparation;
pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) use commit_preparation::prepare_application_commit;

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
        workflow_approval_authority,
    } = current;
    let Some(outcome_identity) =
        super::super::super::WorthQueryApplicationCommitOutcomeIdentity::mint()
    else {
        return terminal(denied(DenialStage::ProposalBinding));
    };
    let (provider, workflow_settlement_publication) =
        match provider.stage_local_workflow_settlement(&admission, outcome_identity) {
            Ok(prepared) => prepared,
            Err(denial) => {
                return terminal(WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::workflow_settlement_denied(&denial),
                ))
            }
        };
    let authorization = match take_commit_authorization(application, &mut admission) {
        Ok(authorization) => authorization,
        Err(outcome) => return terminal(outcome),
    }
    .with_workflow_approval_authority(workflow_approval_authority);
    let Some(mutation_partition) = application.issue_application_mutation_partition() else {
        return terminal(denied(DenialStage::ProposalBinding));
    };
    let effect_posture = provider.effect_posture;
    let provider_attempt =
        match prepare_application_provider_attempt(&lease, provider, mutation_partition) {
            Ok(prepared) => prepared,
            Err(denial) => {
                return terminal(WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::effect_lowering_denied(&denial),
                ))
            }
        };
    WorthQueryApplicationCommitPreparation::Ready(WorthQueryPreparedApplicationCommit {
        admission,
        lease,
        provider_attempt,
        outcome_identity,
        workflow_settlement_publication,
        effect_posture,
        authorization,
        idempotency,
        aftermath_causality,
    })
}

mod provider_attempt;
use provider_attempt::prepare_application_provider_attempt;

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
        .bind_operation(admission.operation_definition_identity())
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

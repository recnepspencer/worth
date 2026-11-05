use crate::domain_computation::primary_graph::application_attempt::mutation_partition::MutationPartitionScopedOps;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
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
    snapshot_lease::WorthQueryApplicationSnapshotLease,
};
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

pub(crate) mod entry_custody;
pub(in crate::domain_computation::primary_graph) use entry_custody::{
    CurrentCommitProgram, ReadmittedCommitProgram,
};
pub(crate) mod currentness;
mod domain_value_write;
pub(crate) mod local_workflow_settlement;
pub(crate) mod native_reference_invariants;
use currentness::{
    readmit_current_basis, select_current_product, validate_elevation_currentness,
    validate_operation_currentness, validate_workflow_deadline,
};
pub(crate) mod running;
pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) use local_workflow_settlement::LocalWorkflowSettlementPublication;

pub struct WorthQueryPreparedApplicationCommit<Schema, Operation, Input, Scope> {
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
    native_reference_invariants:
        worth_query_installation::facade::WorthQueryInvariantExecutionContract,
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

pub enum WorthQueryApplicationCommitPreparation<Schema, Operation, Input, Scope> {
    Ready(WorthQueryPreparedApplicationCommit<Schema, Operation, Input, Scope>),
    Terminal(WorthQueryApplicationCommitOutcome),
}

pub struct WorthQueryApplicationCommitPreparationRequest<
    'a,
    'phase,
    'request,
    Schema,
    Operation,
    Input,
    Scope,
> {
    entry: CurrentCommitProgram<
        'a,
        'phase,
        'request,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
    aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
}

impl<'a, 'phase, 'request, Schema, Operation, Input, Scope>
    WorthQueryApplicationCommitPreparationRequest<
        'a,
        'phase,
        'request,
        Schema,
        Operation,
        Input,
        Scope,
    >
{
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn new(
        entry: CurrentCommitProgram<
            'a,
            'phase,
            'request,
            Schema,
            WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
        aftermath_causality: Option<WorthQueryPendingAftermathCausality>,
    ) -> Self {
        Self {
            entry,
            idempotency,
            elevation_currentness,
            aftermath_causality,
        }
    }
}

struct WorthQueryProviderAttemptPreparation {
    workflow_origin: Option<crate::domain_computation::primary_graph::application_attempt::workflow::operation_output::WorkflowOperationOrigin>,
    local_action_settlement: Option<super::super::super::workflow_transition_program::LocalActionSettlementProof>,
    body_external_emission: Option<crate::domain_computation::primary_graph::operation_body::emission::BodyExternalEffectSelection>,
    required_output_demand: Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
    installed_read_scopes: crate::domain_computation::primary_graph::application_attempt::CompletedReadScopes,
    effect_routes: super::super::super::CompletedFactRoutes,
    application_projection: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedFactProjection>>,
    original_read: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_attempt::CompletedOriginalRead>>,
    consumed_outputs: Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    effects: Vec<WorthQueryApplicationRealizedEffect>,
    content_attachments: worth_relational::facade::content::RelationalContentAttachments,
    application_effect_count: usize,
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
    // Last to drop: physical backing releases before its retained grant.

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

fn prepare_authorized_application_commit<Schema, Operation, Input, Scope>(
    application: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    current: WorthQueryCurrentApplicationCommit<Schema, Operation, Input, Scope>,
) -> WorthQueryApplicationCommitPreparation<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
{
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
    let Some(mutation_partition) = application
        .application_runtime()
        .issue_application_mutation_partition()
    else {
        return terminal(denied(DenialStage::ProposalBinding));
    };
    let effect_posture = provider.effect_posture;
    let native_reference_invariants = match native_reference_invariants::derive(
        &application.provider_graph().layout,
        application.installed_schema().invariants(),
        &provider.effects,
    ) {
        Ok(requirements) => requirements,
        Err(detail) => {
            return terminal(denied_with_detail(DenialStage::InvariantExecution, detail))
        }
    };
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
        native_reference_invariants,
        authorization,
        idempotency,
        aftermath_causality,
    })
}

mod provider_attempt;
use provider_attempt::prepare_application_provider_attempt;

fn take_commit_authorization<Schema, Operation, Input, Scope>(
    application: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
) -> Result<WorthQueryProviderCommitAuthorization, WorthQueryApplicationCommitOutcome> {
    admission
        .take_authorization_dependencies(application.authorization().bridge())
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
        .bind_admission(admission)
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

mod court_doors_for_test;
pub use entry_custody::EntryCustodyDoorsOps;
pub use entry_custody::EntryCustodyEntryCustodyDoorsOps;
pub(in crate::domain_computation) use entry_custody::EntryCustodyScopedOps;

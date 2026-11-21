//! Object-safe runtime ports allowing `attempt` types to retain inherent public
//! methods when invoked with `&WorthQueryPrimaryGraphApplicationRuntime<Schema>`
//! or `&WorthQuerySelectedProductOperation<'_, Schema>` via unsize coercion.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema,
};

use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationAttemptDenial,
};

/// Upper-layer runtime port for `WorkflowOperationAuthority::validate_before_handler`.
pub trait WorkflowOperationAuthorityRuntimePort<Schema>
where
    Schema: ApplicationSchema,
{
    fn validate_workflow_operation_authority(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
        runtime_authority: u64,
        observation: &WorthQueryProductBranchReadIdentity,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;
}

/// Upper-layer port for `WorthQuerySelectedProductOperation<'_, Schema>` used by
/// `WorkflowCapturedInputs::prepare`.
pub trait SelectedProductOperationPort<Schema>
where
    Schema: ApplicationSchema,
{
    fn authority_identity_u64(&self) -> u64;
    fn branch_read_identity(&self) -> WorthQueryProductBranchReadIdentity;
}

/// Upper-layer runtime port for `WorthQueryApplicationEffectProgram` original-input
/// retention (`retain_original_mutation_input`, `retain_performed_correction_input`,
/// `retain_admitted_source_input`).
pub trait EffectProgramOriginalInputRuntimePort<Schema>
where
    Schema: ApplicationSchema,
{
    fn authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn installed_schema(&self) -> &WorthQueryInstalledApplicationSchema<Schema>;
    fn execution_request_for_phase<'phase>(
        &self,
        phase: &'phase WorthQueryAdvancementPhase<'_>,
    ) -> Result<worth_execution::ExecutionRequest<'phase, 'phase>, WorthQueryApplicationAttemptDenial>;
}

/// Upper-layer runtime port for `WorthQueryApplicationEffectProgram::producer_idempotency_identities`.
pub trait ProducerIdentityRuntimePort {
    fn primary_provider(&self) -> &WorthQueryPrimaryGraphProvider;
    fn posts_settlement(
        &self,
        settlement: &crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    ) -> Result<bool, ()>;
}

/// Upper-layer runtime port exposing the runtime authority identity for
/// `WorthQueryApplicationAttemptBasis::capture`.
pub trait RuntimeAuthorityPort {
    fn runtime_authority_identity(&self) -> WorthQueryRuntimeAuthorityIdentity;
}

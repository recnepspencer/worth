//! Principal-first capability admission, before any workflow existence lookup.
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadBinding;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledWorkflowReadBinding;

/// No absence/status payload is carried by an admission refusal.
#[derive(Debug)]
pub enum WorthQueryWorkflowReadDenial {
    PolicyMismatch,
    SubscriptionCapacity,
    Work(worth_execution::WorkCeilingDenial),
    ContinuationMismatch,
    Page(worth_relational::facade::indexes::BoundedRelatedEntityPrefixDenial),
    PageShape(worth_relational::facade::indexes::BoundedRelatedEntityOrderedLookupDenial),
    Reconstruction(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
    PinnedRead(worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial),
    Installation(worth_query_installation::facade::WorthQueryApplicationWorkflowInstallationDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Preparation(worth_relational::facade::mvcc::CompanionPreflightStop),
    Index(worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind),
    SelectedIndexUnavailable,
    BasisUnavailable,
    Corrupt,
    UnsupportedInstanceProtocol(u64),
    Relational(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Phase(crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase),
    Inspection(crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial),
    Execution(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
}

pub(in crate::domain_computation::primary_graph) fn admit<Schema, Spec, B>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    installed: &WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, B>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    input: B::Input,
    request: &WorthQueryRequestScope,
) -> Result<
    WorthQueryAdmittedApplicationCapabilityAccess<Schema, B::Capability, B::Operation, B::Input>,
    WorthQueryWorkflowReadDenial,
>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    B: ApplicationWorkflowReadBinding<Schema>,
{
    use WorthQueryWorkflowReadDenial as Denial;
    // This inspects installed activation metadata only, never workflow instances or indexes.
    let program = selected
        .inspect_selected_program()
        .map_err(Denial::Inspection)?;
    if program.revision() != installed.workflow().program_revision()
        || selected.application().installed_schema().binding_identity()
            != *installed.workflow().schema_binding()
    {
        return Err(Denial::PolicyMismatch);
    }
    let execution = phase
        .execution_request_for(&selected.application().product_runtime)
        .map_err(Denial::Phase)?;
    // The selected native principal owner prepares only its installed identity index.
    let principal = selected
        .resolve_authenticated_principal_issued(
            &execution,
            installed.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
            None,
        )
        .map_err(Denial::Principal)?;
    let access = selected
        .admit_capability_access(
            phase,
            principal.principal(),
            installed.capability(),
            input,
            request,
        )
        .map_err(Denial::Authorization)?;
    access
        .validate_workflow_read_authority(&execution, selected.application())
        .map_err(Denial::Authorization)?;
    Ok(access)
}

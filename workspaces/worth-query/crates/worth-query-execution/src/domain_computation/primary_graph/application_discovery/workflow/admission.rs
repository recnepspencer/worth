//! Principal-first capability admission, before any workflow existence lookup.
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::admitted_access::AdmittedCapabilityAccessRuntimeExt as _;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadBinding;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledWorkflowReadBinding;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_discovery::workflow::denial::WorthQueryWorkflowReadDenial;

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
        .execution_request_for(&selected.application().inner.runtime.product_runtime)
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
        .validate_workflow_read_authority(&execution, selected.application().inner())
        .map_err(Denial::Authorization)?;
    Ok(access)
}

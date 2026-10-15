//! Governed retained-data history. Current authority precedes all instance reads.
use super::workflow_instance_observation::{
    observe_read_only_history, original_scope, ReadOnlyWorkflowHistory,
};
use super::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::application_discovery::{
    admit_workflow_read, WorkflowReadAdmissionDenial,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationWorkflowReadBinding, ApplicationWorkflowReadRoute, ApplicationWorkflowSpec,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledWorkflowReadBinding;
use worth_relational::facade::identity::EntityId;

/// Internal read of an already named target. A public caller cannot supply scope authority.
pub(in crate::domain_computation::primary_graph) fn observe_governed_instance_history<
    Schema,
    Spec,
    B,
>(
    current: &WorthQuerySelectedProductOperation<'_, Schema>,
    data: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    installed: &WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, B>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    input: B::Input,
    request: &WorthQueryRequestScope,
    instance: &PublishedWorkflowInstanceRef,
    lineage: EntityId,
    admission: &mut InvalidationEditAdmission,
) -> Result<ReadOnlyWorkflowHistory, WorkflowReadAdmissionDenial>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    B: ApplicationWorkflowReadBinding<Schema>,
{
    use WorkflowReadAdmissionDenial as Denial;
    // Target/lineage are descriptive native locators, not admission. No target is read first.
    if installed.route() != ApplicationWorkflowReadRoute::Subject {
        return Err(Denial::PolicyMismatch);
    }
    let access = admit_workflow_read(
        current, phase, installed, principal, input, request, admission,
    )?;
    if !std::ptr::eq(current.application(), data.application())
        || current.product().branch_identity() != data.product().branch_identity()
    {
        return Err(Denial::PolicyMismatch);
    }
    let execution = phase
        .execution_request_for(&current.application().product_runtime)
        .map_err(Denial::Phase)?;
    let graph = current
        .application()
        .runtime
        .primary_graph()
        .ok_or(Denial::PolicyMismatch)?;
    let snapshot = data.application_basis().snapshot_handle();
    let resources = installed.workflow().resources();
    graph
        .integration_handle()
        .with_runtime(&execution, |runtime| {
            let layout = graph.layout().workflow();
            let mut facts = Vec::new();
            original_scope(
                &execution,
                runtime,
                snapshot,
                layout,
                instance,
                access.resource_entity_id(),
                lineage,
                &mut facts,
            )
            .map_err(Denial::Execution)?;
            observe_read_only_history(
                &execution,
                runtime,
                snapshot,
                layout,
                instance.entity_id(),
                resources.maximum_retained_transitions_per_instance() as usize,
                resources.history_reconstruction_budget(),
            )
            .map_err(Denial::Execution)
        })
}

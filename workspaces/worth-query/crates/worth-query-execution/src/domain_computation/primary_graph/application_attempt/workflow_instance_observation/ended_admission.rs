//! Terminal refusal at the admitted instance/subject boundary, before compilation.
use super::super::snapshot_lease::WorthQueryApplicationSnapshotLease;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    /// This method consumes no key and runs no authored definition. The existing
    /// admitted request supplies the subject; an unrelated subject sees no state.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn refuse_ended_workflow_instance(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        instance: &PublishedWorkflowInstanceRef,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        refuse_ended_on_subject(
            &self.lease,
            self.admission.scope_entity_id(),
            execution,
            instance,
        )
    }
}

fn refuse_ended_on_subject(
    lease: &WorthQueryApplicationSnapshotLease,
    subject: EntityId,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    instance: &PublishedWorkflowInstanceRef,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if instance.branch() != lease.product().product_branch() {
        return Err(denial("workflow instance belongs to another branch"));
    }
    let layout = lease.layout.workflow();
    lease.handle().with_runtime(execution, |runtime| {
        let snapshot = lease.snapshot();
        let state = observe_field_value(
            runtime,
            execution,
            snapshot,
            instance.entity_id(),
            layout.instance.entity_kind,
            &layout.instance.state,
        )?;
        let Some(ended) = instance_binding::ended_state_denial(&state) else {
            return Ok(());
        };
        let mut facts = Vec::new();
        exact(
            execution,
            runtime,
            snapshot,
            instance.entity_id(),
            layout.instance.entity_kind,
            &layout.instance.branch_occurrence,
            AspectValue::UInt64(instance.started_occurrence()),
            &mut facts,
        )?;
        subject::observe_subject(
            execution,
            runtime,
            snapshot,
            layout,
            instance.entity_id(),
            subject,
            &mut facts,
        )?;
        actors::observe(
            execution,
            runtime,
            snapshot,
            layout,
            instance.entity_id(),
            &mut facts,
        )?;
        Err(ended)
    })
}

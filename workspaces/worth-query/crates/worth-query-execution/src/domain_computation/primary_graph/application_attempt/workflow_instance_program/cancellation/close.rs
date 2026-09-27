//! A live instance ends where it stands: it records the cancellation and
//! releases its live membership, and reports every effect it performed.

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::super::super::workflow_instance_observation::{
    instance_binding::owner_custody, observe_workflow_instance, WorkflowInstanceObservationPurpose,
};
use super::super::super::{
    PublishedWorkflowDefinitionRef, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
    WorthQueryApplicationRealizedEffect, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};
use super::super::{performed, PublishedWorkflowInstanceRef};
use super::HISTORY;
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::{WorkflowInstanceState, WorkflowPerformedEffect},
    schema::WorthQueryWorkflowLayout,
};

/// The writes that end a live instance, what it performed, and the facts
/// they rest on.
pub(super) struct ClosedWorkflowInstance {
    pub(super) effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub(super) performed: Vec<WorkflowPerformedEffect>,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

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
    /// Reads the live instance under the definition the installed program
    /// runs and closes it; an instance that already ended is refused.
    pub(super) fn close_live_workflow_instance<Spec, Program>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        layout: &WorthQueryWorkflowLayout,
        instance: &PublishedWorkflowInstanceRef,
        identity: &str,
    ) -> Result<ClosedWorkflowInstance, WorthQueryApplicationAttemptDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let handle = self.lease.handle();
        let snapshot = self.lease.snapshot();
        let (mut compiled, mut facts) = reconstruct_compiled_definition(
            handle,
            snapshot,
            layout,
            &PublishedWorkflowDefinitionRef::retained(
                instance.branch(),
                instance.definition_entity_id(),
                instance.definition_content_identity().clone(),
            ),
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        let maximum_transitions = super::maximum_transitions(installed);
        let budget = installed.resources().history_reconstruction_budget();
        let entity = instance.entity_id();
        handle.with_runtime(|runtime| {
            let mut observed = observe_workflow_instance(
                handle,
                runtime,
                snapshot,
                layout,
                instance,
                self.admission.scope_entity_id(),
                compiled.lineage(),
                &mut compiled,
                maximum_transitions,
                budget,
                WorkflowInstanceObservationPurpose::Close,
            )?;
            let Some(live_membership) = observed.live_membership else {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCompleted,
                    self.admission.operation(),
                ));
            };
            observed.ensure_history(
                handle,
                runtime,
                snapshot,
                layout,
                entity,
                maximum_transitions,
                &compiled,
            )?;
            facts.append(&mut observed.facts);
            // An external operation the owner still holds has committed but
            // not settled. Cancelling now would dispose of that custody.
            let (custody, custody_fact) = owner_custody(runtime, snapshot, layout, entity)?;
            if custody.is_some_and(|transition| {
                !observed
                    .transitions
                    .iter()
                    .any(|settled| settled.identity == transition)
            }) {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
                    self.admission.operation(),
                ));
            }
            facts.push(custody_fact);
            // A live instance never recorded a cancellation; the write below
            // replaces that absence.
            facts.push(WorthQueryApplicationObservedFact::AbsentField {
                entity_id: entity,
                kind: layout.instance.entity_kind,
                locator: layout.instance.cancellation_identity.clone(),
            });
            let effects = vec![
                WorthQueryApplicationRealizedEffect::UpdateEntity {
                    entity: "workflow-instance".to_owned(),
                    entity_id: entity,
                    fields: BTreeMap::from([
                        (
                            layout.instance.state.clone(),
                            AspectValue::UInt64(WorkflowInstanceState::Cancelled.persisted_tag()),
                        ),
                        (
                            layout.instance.cancellation_identity.clone(),
                            AspectValue::String(InternedString::Raw(identity.to_owned())),
                        ),
                    ]),
                },
                WorthQueryApplicationRealizedEffect::DeleteRelation {
                    relation_id: live_membership,
                },
            ];
            let mut performed = performed::own_effects(&observed.transitions, &compiled, HISTORY)?;
            performed.extend(performed::inherited_effects(
                runtime,
                snapshot,
                layout,
                entity,
                maximum_transitions,
                &mut facts,
                HISTORY,
            )?);
            Ok(ClosedWorkflowInstance {
                effects,
                performed,
                facts,
            })
        })
    }
}

use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::workflow::{
    definition::{reconstruct_compiled_definition, WorkflowDefinitionCompilationPosture},
    instance::select_current_transition,
};

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn is_actor_permission_denial(denial: &WorthQueryOperationAuthorizationDenial) -> bool {
        actor_permission_denial(denial)
    }

    /// Observe a denied actor's exact live head. Failure leaves the original
    /// admission denial intact; an instance reference is never observation authority.
    pub fn observe_awaiting_actor<Capability, Operation, Schema, Scope, Spec, Program>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: &PublishedWorkflowInstanceRef,
        subject: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        denial: &WorthQueryOperationAuthorizationDenial,
    ) -> Option<super::super::RequiredWorkflowActor>
    where
        Capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema> + 'static,
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if !Self::is_actor_permission_denial(denial)
            || !installed.advance_binding_matches::<Capability, Operation>()
            || instance.branch() != selected.product().product_branch()
            || installed.schema_binding()
                != &selected.application().installed_schema().binding_identity()
            || selected.inspect_selected_program().ok()?.revision() != installed.program_revision()
        {
            return None;
        }
        let graph = selected.application().runtime.primary_graph()?;
        let handle = graph.integration_handle();
        let lease = WorthQueryApplicationSnapshotLease::acquire(
            handle,
            graph.retain_layout(),
            selected.product().retained_clone(),
        )
        .ok()?;
        let layout = graph.layout().workflow();
        let published =
            crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef::retained(
                instance.branch(),
                instance.definition_entity_id(),
                instance.definition_content_identity().clone(),
            );
        let (compiled, _) = reconstruct_compiled_definition(
            lease.handle(),
            lease.snapshot(),
            layout,
            &published,
            instance.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )
        .ok()?;
        let observed = lease.handle().with_runtime(|runtime| {
            crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::observe_workflow_instance(
                lease.handle(), runtime, lease.snapshot(), layout, instance, subject.entity_id(),
                compiled.lineage(), &compiled,
                usize::try_from(installed.resources().maximum_retained_transitions_per_instance()).unwrap_or(usize::MAX),
                installed.resources().history_reconstruction_budget(),
            )
        }).ok()?;
        observed.live_membership?;
        let selected_transition =
            select_current_transition(&compiled, instance.entity_id(), &observed.progress_basis)
                .ok()?;
        Some(super::super::RequiredWorkflowActor::new(
            instance.entity_id(),
            denial.clone(),
            &selected_transition,
        ))
    }
}

fn actor_permission_denial(denial: &WorthQueryOperationAuthorizationDenial) -> bool {
    use WorthQueryOperationAuthorizationDenialKind as Kind;
    !denial.causes().is_empty()
        && denial.causes().iter().all(|kind| {
            matches!(
                kind,
                Kind::CapabilityGrantMissing
                    | Kind::CapabilityAuthorizationMissing
                    | Kind::ExplicitDenyRuleMatched
                    | Kind::ConflictRuleMatched
                    | Kind::SeparationOfDutyRuleMatched
                    | Kind::DistinctActorRuleMatched
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_actor_permission_denials_can_become_actor_waits() {
        use WorthQueryOperationAuthorizationDenialKind as Kind;
        for kind in [
            Kind::CapabilityGrantMissing,
            Kind::ExplicitDenyRuleMatched,
            Kind::SeparationOfDutyRuleMatched,
        ] {
            assert!(actor_permission_denial(
                &WorthQueryOperationAuthorizationDenial::new(kind, "actor")
            ));
        }
        for kind in [
            Kind::Cancelled,
            Kind::DeadlineExceeded,
            Kind::ExpiredAuthentication,
            Kind::CapabilityExpired,
            Kind::PurposeMismatch,
            Kind::PermissionDenied,
            Kind::ScopeMismatch,
            Kind::RelationalObservationRejected,
        ] {
            assert!(!actor_permission_denial(
                &WorthQueryOperationAuthorizationDenial::new(kind, "not actor permission")
            ));
        }
    }
}

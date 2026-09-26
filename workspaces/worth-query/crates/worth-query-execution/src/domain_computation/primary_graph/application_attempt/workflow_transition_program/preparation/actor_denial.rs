use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

use super::*;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

impl WorthQueryWorkflowAdvanceAdapter {
    /// An actor permission denial can identify the blocked posture, but never
    /// authorizes reading the workflow's live head.
    pub fn redacted_awaiting_actor<Capability, Operation, Schema, Spec, Program>(
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        instance: &PublishedWorkflowInstanceRef,
        denial: &WorthQueryOperationAuthorizationDenial,
    ) -> Option<super::super::RequiredWorkflowActor>
    where
        Capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema> + 'static,
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if !actor_permission_denial(denial)
            || !installed.advance_binding_matches::<Capability, Operation>()
        {
            return None;
        }
        Some(super::super::RequiredWorkflowActor::new(
            instance.entity_id(),
            denial.clone(),
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

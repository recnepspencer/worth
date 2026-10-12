use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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
    pub fn redacted_awaiting_actor<Capability, Operation, Schema, Spec>(
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
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
                Kind::Refused(AuthorizationRefusal::CapabilityGrantMissing)
                    | Kind::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing)
                    | Kind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched)
                    | Kind::Refused(AuthorizationRefusal::ConflictRuleMatched)
                    | Kind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched)
                    | Kind::Refused(AuthorizationRefusal::DistinctActorRuleMatched)
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
            Kind::Refused(AuthorizationRefusal::CapabilityGrantMissing),
            Kind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched),
            Kind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched),
        ] {
            assert!(actor_permission_denial(
                &WorthQueryOperationAuthorizationDenial::new(kind, "actor")
            ));
        }
        for kind in [
            Kind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
            )),
            Kind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
            )),
            Kind::Refused(AuthorizationRefusal::ExpiredAuthentication),
            Kind::Refused(AuthorizationRefusal::CapabilityExpired),
            Kind::Refused(AuthorizationRefusal::PurposeMismatch),
            Kind::Refused(AuthorizationRefusal::PermissionDenied),
            Kind::Refused(AuthorizationRefusal::ScopeMismatch),
            Kind::Refused(AuthorizationRefusal::RelationalObservationRejected),
        ] {
            assert!(!actor_permission_denial(
                &WorthQueryOperationAuthorizationDenial::new(kind, "not actor permission")
            ));
        }
    }
}

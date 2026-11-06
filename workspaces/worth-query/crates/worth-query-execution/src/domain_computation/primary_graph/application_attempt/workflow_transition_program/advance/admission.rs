//! Ordinary progression requires the installed advance capability, not a business-action admission.
use {
    super::denial, super::ApplicationCapabilityMarkerIdentity,
    super::ApplicationOperationMarkerIdentity, super::ApplicationSchema,
    super::ApplicationWorkflowSpec, super::WorthQueryCompleteApplicationReadSet,
    super::WorthQueryInstalledApplicationWorkflowSpec,
    super::WorthQueryProjectedApplicationMutation,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
    crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind,
};

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
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn authorize_advance<
        Capability,
        Spec,
    >(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if !installed.advance_binding_matches::<Capability, Operation>()
            || self.admission.installed_capability_identity()
                != Some(*installed.advance_capability_identity_bytes())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                self.admission.operation(),
            ));
        }
        Ok(())
    }
}

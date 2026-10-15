//! Exact operation affinity is shared by frontier and requested action routes.
use super::InstalledWorkflowOperation;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{ApplicationWorkflowOperationRef, ApplicationWorkflowSpec},
};
impl InstalledWorkflowOperation {
    pub(super) fn declared<Spec, Binding>(supports_local_workflow_action: bool) -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationMutationBinding<Spec::Schema>,
    {
        let reference = ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>();
        Self {
            marker: reference.operation_type(),
            identifier: reference.identifier(),
            input_type: reference.input_type().clone(),
            binding_type: std::any::TypeId::of::<Binding>(),
            binding_identity: Binding::IDENTITY,
            requires_workflow_authority: Binding::REQUIRES_WORKFLOW_AUTHORITY,
            supports_local_workflow_action,
            reference,
            inbound_ref: None,
        }
    }
    pub(in crate::application_program::workflow) fn matches_reference(
        &self,
        operation: &ApplicationWorkflowOperationRef,
        requires_authority: bool,
    ) -> bool {
        self.marker == operation.operation_type()
            && self.identifier == operation.identifier()
            && &self.input_type == operation.input_type()
            && self.requires_workflow_authority == requires_authority
            && self.reference.workflow_execution_requirements()
                == operation.workflow_execution_requirements()
            && match operation.binding() {
                Some((identity, marker, posture)) => {
                    self.binding_type == marker
                        && self.binding_identity == identity
                        && self.requires_workflow_authority == posture
                }
                None => !requires_authority,
            }
    }
}
impl<Schema, Spec> super::WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    #[doc(hidden)]
    pub fn supports_local_workflow_action<Binding: ApplicationMutationBinding<Schema>>(
        &self,
    ) -> bool {
        let reference = ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>();
        self.operations
            .iter()
            .filter(|operation| {
                operation.supports_local_workflow_action
                    && operation.matches_reference(&reference, true)
            })
            .count()
            == 1
    }
}

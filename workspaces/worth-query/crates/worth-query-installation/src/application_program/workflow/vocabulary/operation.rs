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
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<Spec::Schema>,
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
            admission_identity: None,
            capability: <Binding::Admission as worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionMode>::capability_identity(),
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
                        && self.reference.is_workflow_control() == operation.is_workflow_control()
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
    /// Descriptive identity of the installed local binding's sealed admission mode.
    /// `Some(None)` denotes conventional admission; no result grants permission.
    #[doc(hidden)]
    pub fn local_action_admission_identity<
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<
            Schema,
        >,
    >(
        &self,
    ) -> Option<Option<[u8; 32]>> {
        let reference = ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>();
        use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionMode;
        let mut matches = self.operations.iter().filter(|operation| {
            operation.supports_local_workflow_action
                && operation.matches_reference(&reference, true)
                && operation.capability == Binding::Admission::capability_identity()
        });
        let operation = matches.next()?;
        matches
            .next()
            .is_none()
            .then_some(operation.admission_identity)
    }
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

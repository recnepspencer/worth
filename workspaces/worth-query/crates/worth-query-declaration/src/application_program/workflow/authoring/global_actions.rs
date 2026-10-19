use super::ApplicationWorkflowDefinitionBuilder;
use crate::{
    application_operation::ApplicationMutationBinding,
    application_program::workflow::{
        ApplicationWorkflowGlobalAction, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
    },
};

impl<Spec: ApplicationWorkflowSpec> ApplicationWorkflowDefinitionBuilder<Spec> {
    /// Declare one independently authorized business termination for every live frontier.
    /// The body must be local, unsigned, occurrence-bound and explicitly approval-free.
    pub fn termination_action<Binding: ApplicationMutationBinding<Spec::Schema>>(
        &mut self,
    ) -> &mut Self {
        self.global_actions
            .push(ApplicationWorkflowGlobalAction::Terminate(
                ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>(),
            ));
        self
    }
}

//! Workflow registration held in the program runtime shared by Home sessions.
use super::*;
use std::sync::Arc;
use WorthQueryWorkflowRuntimeBindingDenial as Denial;

impl<Schema: ApplicationSchema + 'static, Program>
    WorthQueryProgramApplicationRuntime<Schema, Program>
{
    /// Retain a workflow configuration in this installation before making session handles.
    pub fn register_home_workflow<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        authentication: WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Result<
        (),
        (
            WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            WorthQueryAuthenticationEventSigningOwner<Schema>,
            Denial,
        ),
    > {
        if self.backing.workflow.is_some() {
            return Err((workflow, authentication, Denial::AlreadySupported));
        }
        let configuration = self.install_workflow_configuration(workflow, authentication)?;
        Arc::get_mut(&mut self.backing)
            .expect("registration required unique runtime custody")
            .workflow = Some(Box::new(configuration));
        Ok(())
    }

    fn retained_workflow<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
    ) -> Option<&WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>> {
        self.backing.workflow.as_ref()?.downcast_ref()
    }

    pub fn workflow_vocabulary<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
    ) -> Option<WorthQueryWorkflowVocabulary<'_, Schema, Spec>> {
        self.retained_workflow()?.vocabulary_for(self).ok()
    }

    pub fn retained_workflow_spec<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>> {
        Some(self.retained_workflow()?.workflow_spec())
    }

    pub fn support_home_workflow<Spec: ApplicationWorkflowSpec<Schema = Schema>>(
        &mut self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<
        (),
        (
            WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
            Denial,
        ),
    > {
        if self.retained_workflow::<Spec>().is_none() {
            return Err((workflow, Denial::WorkflowSpecNotRetained));
        }
        if Arc::get_mut(&mut self.backing).is_none() {
            return Err((workflow, Denial::RuntimeKeepaliveRetained));
        }
        let configuration = self
            .retained_workflow::<Spec>()
            .expect("configuration was checked");
        if let Err(denial) = configuration.admit_support(self, &workflow) {
            return Err((workflow, denial));
        }
        let backing =
            Arc::get_mut(&mut self.backing).expect("registration required unique runtime custody");
        let configuration = backing
            .workflow
            .as_mut()
            .and_then(|configuration| {
                configuration.downcast_mut::<WorthQueryWorkflowRuntimeConfiguration<Schema, Spec>>()
            })
            .expect("configuration was checked");
        if let Err(denial) = registration::install_workflow_registration(
            backing
                .runtime
                .as_mut()
                .expect("only the live Query owns runtime access"),
            &workflow,
            &configuration.authentication,
        ) {
            return Err((workflow, denial));
        }
        configuration.supported.push(workflow);
        Ok(())
    }
}

//! Register the actual program member, never a second executable-body registry.
use super::{
    denial, ApplicationSchema, ApplicationWorkflowSpec, InstalledWorkflowOperation, TypeId,
    WorthQueryApplicationWorkflowInstallationDenial,
    WorthQueryApplicationWorkflowInstallationDenialKind,
    WorthQueryApplicationWorkflowSpecInstallation,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
impl<'installed, Schema, Spec, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn operation<Binding>(
        mut self,
    ) -> Result<Self, WorthQueryApplicationWorkflowInstallationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding<Schema>,
    {
        self.insert_marker(0, TypeId::of::<Binding>(), Binding::IDENTITY)?;
        if !self
            .program
            .actions()
            .iter()
            .any(|action| action.ordinary_mutation_binding_type() == Some(TypeId::of::<Binding>()))
        {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInProgram,
                Binding::IDENTITY,
            ));
        }
        let installed = self
            .schema
            .installed_mutation_binding::<Binding>()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInstalled,
                    Binding::IDENTITY,
                )
            })?;
        if Binding::WORKFLOW_CONTROL && installed.operation().body().is_some() {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInstalled,
                Binding::IDENTITY,
            ));
        }
        if installed
            .operation()
            .body()
            .is_some_and(|body| body.succession().is_some())
        {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::SuccessionBindingNotInstalled,
                Binding::IDENTITY,
            ));
        }
        if installed.operation().body().is_some_and(|body| {
            body.creation()
                .is_some_and(|creation| creation.enrollment().is_some())
        }) {
            return Err(denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::BirthBindingNotInstalled,
                Binding::IDENTITY,
            ));
        }
        let supports_local_workflow_action = Binding::REQUIRES_APPLICATION_PROGRAM
            && Binding::REQUIRES_WORKFLOW_AUTHORITY
            && !Binding::WORKFLOW_CONTROL
            && installed.operation().body().is_some_and(|body| {
                body.creation()
                    .is_none_or(|creation| creation.signing().is_none())
            });
        let mut operation =
            InstalledWorkflowOperation::declared::<Spec, Binding>(supports_local_workflow_action);
        if let Some(key) =
            crate::application_capability::ApplicationCapabilityRegistryKey::from_mutation_admission::<
                Schema,
                Binding,
            >()
        {
            operation.admission_identity = Some(*self.schema.capability_registry.get(&key)
                .ok_or_else(|| denial(WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInstalled,
                    Binding::IDENTITY))?.identity().bytes());
        }
        operation.inbound_ref = self
            .schema
            .installed_workflow_inbound_ref(Binding::Operation::IDENTIFIER);
        self.operations.push(operation);
        Ok(self)
    }
}

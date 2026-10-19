//! Enrollment membership binds the actual creator, output and own capability.
use super::{
    denial, ApplicationSchema, ApplicationWorkflowSpec, InstalledWorkflowBirth, TypeId,
    WorthQueryApplicationWorkflowInstallationDenial as Denial,
    WorthQueryApplicationWorkflowInstallationDenialKind as Kind,
    WorthQueryApplicationWorkflowSpecInstallation,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationScopeBinding, WorthQueryApplicationDeclaredOutputRole,
        WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    },
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding},
};

impl<Schema, Spec, Program> WorthQueryApplicationWorkflowSpecInstallation<'_, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub fn birth_operation<Binding, Role>(mut self) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema> + ApplicationCapabilityMutationBinding<Schema>,
        Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
        <Binding::Operation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            ApplicationStructuredValueBinding<Value = Binding::Input>,
        Binding::Input: ApplicationCapabilityRequest<
            Schema,
            Binding::Capability,
            Scope = <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryCreateOutput,
        >,
    {
        let denied = || denial(Kind::BirthBindingNotInstalled, Binding::IDENTITY);
        self.insert_marker(7, TypeId::of::<Binding>(), Binding::IDENTITY)?;
        if !self
            .program
            .actions()
            .iter()
            .any(|action| action.ordinary_mutation_binding_type() == Some(TypeId::of::<Binding>()))
        {
            return Err(denial(Kind::OperationNotInProgram, Binding::IDENTITY));
        }
        let installed = self
            .schema
            .installed_mutation_binding::<Binding>()
            .map_err(|_| denied())?;
        let body = installed.operation().body().ok_or_else(denied)?;
        let creation = body.creation().ok_or_else(denied)?;
        let enrollment = creation.enrollment().ok_or_else(denied)?;
        let role = <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR;
        if !Binding::REQUIRES_APPLICATION_PROGRAM
            || Binding::REQUIRES_WORKFLOW_AUTHORITY
            || Binding::WORKFLOW_CONTROL
            || installed
                .operation()
                .contracts()
                .external_effect()
                .is_declared()
            || creation.signing().is_some()
            || enrollment.workflow_spec() != Spec::IDENTITY.as_str()
            || enrollment.binding() != Binding::IDENTITY
            || enrollment.role() != role.name()
            || creation.entity() != role.entity()
            || Binding::workflow_execution_requirements()
                .approval()
                .is_some()
        {
            return Err(denied());
        }
        let capability = self
            .bind_capability::<Binding::Capability, Binding::Operation, Binding::Input>(
                Kind::BirthBindingNotInstalled,
            )?;
        self.births.push(InstalledWorkflowBirth {
            binding_type: TypeId::of::<Binding>(),
            binding_identity: Binding::IDENTITY,
            role_type: TypeId::of::<Role>(),
            role_name: role.name(),
            entity_name: role.entity(),
            capability,
        });
        Ok(self)
    }
}

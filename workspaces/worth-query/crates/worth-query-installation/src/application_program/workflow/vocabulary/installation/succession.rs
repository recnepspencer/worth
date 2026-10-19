//! A completed-chain successor binds one Scope/Preserve body and its own capability.
use super::{
    denial, ApplicationSchema, ApplicationWorkflowSpec, InstalledWorkflowSuccession, TypeId,
    WorthQueryApplicationWorkflowInstallationDenial as Denial,
    WorthQueryApplicationWorkflowInstallationDenialKind as Kind,
    WorthQueryApplicationWorkflowSpecInstallation,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationScopeBinding, WorthQueryApplicationDeclaredOutputRole,
        WorthQueryApplicationOutputRole, WorthQueryPreserveOutput,
    },
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding},
};
impl<Schema: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = Schema>, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'_, Schema, Spec, Program>
{
    pub(super) fn validate_succession_producers(&self) -> Result<(), Denial> {
        use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;
        for member in self.schema.installed_declaration().members() {
            let ApplicationSchemaMember::OperationBody { body, .. } = member else {
                continue;
            };
            let Some(succession) = body.succession() else {
                continue;
            };
            if !self
                .successions
                .iter()
                .any(|binding| binding.binding_identity == succession.binding())
            {
                continue;
            }
            if let Some(matching) = succession.output_match()
                && !self.operations.iter().any(|producer| {
                    producer.binding_identity == matching.producer_binding()
                        && producer.requires_workflow_authority
                })
            {
                return Err(denial(
                    Kind::SuccessionBindingNotInstalled,
                    matching.producer_binding(),
                ));
            }
        }
        Ok(())
    }
    pub fn succession_operation<Binding, Role>(mut self) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema> + ApplicationCapabilityMutationBinding<Schema>,
        Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
        <Binding::Operation as ApplicationOperationMarkerIdentity<Schema>>::InputBinding:
            ApplicationStructuredValueBinding<Value = Binding::Input>,
        Binding::ScopeBinding: ApplicationMutationScopeBinding<Schema, Scope = Role::Entity>,
        Binding::Input:
            ApplicationCapabilityRequest<Schema, Binding::Capability, Scope = Role::Entity>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
    {
        let denied = || denial(Kind::SuccessionBindingNotInstalled, Binding::IDENTITY);
        self.insert_marker(8, TypeId::of::<Binding>(), Binding::IDENTITY)?;
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
        let succession = body.succession().ok_or_else(denied)?;
        let role = <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR;
        if !Binding::REQUIRES_APPLICATION_PROGRAM
            || Binding::REQUIRES_WORKFLOW_AUTHORITY
            || Binding::WORKFLOW_CONTROL
            || installed
                .operation()
                .contracts()
                .external_effect()
                .is_declared()
            || succession.workflow_spec() != Spec::IDENTITY.as_str()
            || succession.binding() != Binding::IDENTITY
            || succession.role() != role.name()
            || body.scope_entity() != Some(role.entity())
            || Binding::workflow_execution_requirements()
                .approval()
                .is_some()
        {
            return Err(denied());
        }
        let capability = self
            .bind_capability::<Binding::Capability, Binding::Operation, Binding::Input>(
                Kind::SuccessionBindingNotInstalled,
            )?;
        self.successions.push(InstalledWorkflowSuccession {
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

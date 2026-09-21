use std::any::TypeId;

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationOperationMarkerIdentity,
};

use super::{InstalledWorkflowCapabilityBinding, WorthQueryInstalledApplicationWorkflowSpec};

impl<Schema, Spec, Program> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>
where
    Schema: super::ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    #[doc(hidden)]
    pub fn approval_binding_matches<Capability, Operation>(&self) -> bool
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    {
        self.approval_binding::<Capability, Operation>().is_some()
    }

    #[doc(hidden)]
    pub fn approval_capability_identity_bytes<Capability, Operation>(&self) -> Option<&[u8; 32]>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    {
        self.approval_binding::<Capability, Operation>()
            .map(|binding| &binding.installed_identity)
    }

    fn approval_binding<Capability, Operation>(&self) -> Option<&InstalledWorkflowCapabilityBinding>
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    {
        self.approvals.iter().find_map(|approval| {
            approval
                .binding
                .matches_binding(
                    TypeId::of::<Capability>(),
                    Capability::IDENTIFIER,
                    &Capability::PORTABLE_TYPE_IDENTITY,
                    TypeId::of::<Operation>(),
                    Operation::IDENTIFIER,
                )
                .then_some(&approval.binding)
        })
    }
}

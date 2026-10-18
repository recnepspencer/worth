//! Exact succession membership is descriptive; native admission owns predecessor legality.
use super::{TypeId, WorthQueryInstalledApplicationWorkflowSpec};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, WorthQueryApplicationOutputRole, WorthQueryPreserveOutput,
    },
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationSchema,
};
impl<Schema: ApplicationSchema, Spec: ApplicationWorkflowSpec<Schema = Schema>>
    WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>
{
    #[doc(hidden)]
    pub fn succession_capability_identity_bytes<Binding, Role>(&self) -> Option<&[u8; 32]>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
    {
        let mut matches = self.successions.iter().filter(|entry| {
            entry.binding_type == TypeId::of::<Binding>()
                && entry.role_type == TypeId::of::<Role>()
                && entry.binding_identity == Binding::IDENTITY
        });
        let first = matches.next()?;
        matches
            .next()
            .is_none()
            .then_some(&first.capability.installed_identity)
    }
}

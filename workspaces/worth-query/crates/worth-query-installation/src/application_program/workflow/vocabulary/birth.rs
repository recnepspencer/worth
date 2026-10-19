//! Exact installed birth membership is descriptive, not runtime start authority.
use super::{TypeId, WorthQueryInstalledApplicationWorkflowSpec};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    },
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationSchema,
};
impl<Schema, Spec> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    #[doc(hidden)]
    pub fn birth_capability_identity_bytes<Binding, Role>(&self) -> Option<&[u8; 32]>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryCreateOutput,
        >,
    {
        let mut matches = self.births.iter().filter(|birth| {
            birth.binding_type == TypeId::of::<Binding>()
                && birth.role_type == TypeId::of::<Role>()
                && birth.binding_identity == Binding::IDENTITY
        });
        let first = matches.next()?;
        matches
            .next()
            .is_none()
            .then_some(&first.capability.installed_identity)
    }
}

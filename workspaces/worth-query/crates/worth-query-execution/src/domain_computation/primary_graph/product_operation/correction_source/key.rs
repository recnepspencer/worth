//! A correction key belongs to the installed caller domain, independently of its target.
use super::*;
use worth_query_declaration::facade::{
    application_aftermath::ApplicationInverseBinding,
    application_operation::{application_value_identity, ApplicationValueIdentityDomain},
};
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::correction_source::WorthQueryCorrectionCommandKey;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn correction_command_key<C: ApplicationCorrectionBinding<Schema>>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        wire_key: &str,
    ) -> Result<WorthQueryCorrectionCommandKey, WorthQueryCorrectionDerivationDenial> {
        self.action_command_key::<ApplicationInverseBinding<Schema, C>>(
            access, phase, principal, request, wire_key,
        )
    }
    /// A history correction's per-attempt command key: the caller's command
    /// key composed with the document's attempt identity (plan §10.1 step
    /// 5), so a retry after a stopped attempt never meets the parked one's
    /// reversal. The caller's idempotency key itself is left untouched, so
    /// structured and numeric keys keep decoding.
    pub fn history_attempt_command_key(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        command: &WorthQueryCorrectionCommandKey,
        attempt: &str,
    ) -> Result<WorthQueryCorrectionCommandKey, WorthQueryCorrectionDerivationDenial> {
        let canonical = application_value_identity(
            ApplicationValueIdentityDomain::RecordedCorrectionKey,
            "history-attempt",
            &(command.identity(), attempt),
        )
        .map_err(WorthQueryCorrectionDerivationDenial::CommandKey)?;
        Ok(WorthQueryCorrectionCommandKey::from_canonical(access, canonical))
    }
    pub fn reapplication_command_key<C: ApplicationCorrectionBinding<Schema>>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        wire_key: &str,
    ) -> Result<WorthQueryCorrectionCommandKey, WorthQueryCorrectionDerivationDenial> {
        self.action_command_key::<C::Source>(access, phase, principal, request, wire_key)
    }
    fn action_command_key<Binding: ApplicationMutationBinding<Schema>>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        wire_key: &str,
    ) -> Result<WorthQueryCorrectionCommandKey, WorthQueryCorrectionDerivationDenial> {
        use WorthQueryCorrectionDerivationDenial as Denial;
        let binding = self
            .application()
            .installed_schema()
            .installed_mutation_binding::<Binding>()
            .map_err(|_| Denial::UnsupportedTargetProgram)?;
        let actor = self.resolve_authenticated_principal(phase, binding.principal_binding(), principal,
            request, crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary)
            .map_err(Denial::Principal)?;
        let entity = actor.principal_entity_id();
        let identity = actor.binding_identity();
        let canonical = application_value_identity(
            ApplicationValueIdentityDomain::RecordedCorrectionKey,
            "installed-caller",
            &(
                identity.package_identity().bytes(),
                identity.schema_identity().bytes(),
                entity.partition_value(),
                entity.local_slot_value(),
                entity.generation_value(),
                wire_key,
            ),
        )
        .map_err(Denial::CommandKey)?;
        Ok(WorthQueryCorrectionCommandKey::from_canonical(
            access, canonical,
        ))
    }
}

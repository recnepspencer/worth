//! A correction key belongs to the installed caller domain, independently of its target.
use super::*;
use worth_query_declaration::facade::{
    application_aftermath::ApplicationInverseBinding,
    application_operation::{
        application_value_identity, ApplicationCanonicalIdentity, ApplicationValueIdentityDomain,
    },
};

/// Canonically encoded client command key. This descriptive value grants no execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionCommandKey {
    canonical: ApplicationCanonicalIdentity,
}
impl WorthQueryCorrectionCommandKey {
    pub fn identity(&self) -> [u8; 32] {
        self.canonical.identity()
    }
    pub fn canonical(&self) -> ApplicationCanonicalIdentity {
        self.canonical
    }
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn correction_command_key<C: ApplicationCorrectionBinding<Schema>>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        wire_key: &str,
    ) -> Result<WorthQueryCorrectionCommandKey, WorthQueryCorrectionDerivationDenial> {
        self.action_command_key::<ApplicationInverseBinding<Schema, C>>(
            phase, principal, request, wire_key,
        )
    }
    pub fn reapplication_command_key<C: ApplicationCorrectionBinding<Schema>>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        wire_key: &str,
    ) -> Result<WorthQueryCorrectionCommandKey, WorthQueryCorrectionDerivationDenial> {
        self.action_command_key::<C::Source>(phase, principal, request, wire_key)
    }
    fn action_command_key<Binding: ApplicationMutationBinding<Schema>>(
        &self,
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
        Ok(WorthQueryCorrectionCommandKey { canonical })
    }
}

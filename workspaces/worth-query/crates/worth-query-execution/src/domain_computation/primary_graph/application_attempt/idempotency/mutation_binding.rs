use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    append_identity_slot as append_optional_identity_slot, WorthQueryApplicationIdempotencyBinding,
};

const DOMAIN: &[u8] = b"worth-query.idempotency-mutation-binding.v1";

fn slot(binding_identity: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    for part in [DOMAIN, binding_identity.as_bytes()] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize().into()
}

impl WorthQueryApplicationIdempotencyBinding {
    /// The idempotency binding of one request to `Binding`, from the identities
    /// its key and input encoded to, so the request is never encoded twice.
    ///
    /// It names `Binding` as the mutation the request runs, so a key reused on
    /// another binding of the same operation is intent drift, not a replay.
    /// Recovery, resolution, and every caller that commits a handler's program
    /// use this door; the commit refuses a binding that does not name the
    /// handler's own mutation or derives its intent from an input other than
    /// the one the handler decided on.
    pub fn for_mutation_identities<Schema, Binding>(
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        Self::new(*identities.key_identity(), *identities.input_identity())
            .bind_mutation::<Schema, Binding>()
    }

    /// Extends drift identity without replacing the handler's own input proof.
    /// Only a canonical full envelope intent can occupy this slot.
    pub fn bind_edit_envelope(
        mut self,
        intent: &worth_query_declaration::facade::application_operation::ApplicationEditIntentIdentity,
    ) -> Self {
        self.envelope_intent = Some(*intent.identity());
        self
    }

    /// A secondary authored stage has identity and session custody, but no
    /// independent durable idempotency record. Its envelope alone owns the key.
    pub(in crate::domain_computation::primary_graph) fn for_authored_stage_identities<S, B>(
        identities: &ApplicationMutationIdentities<'_, S, B>,
    ) -> Option<Self>
    where
        S: ApplicationSchema,
        B: ApplicationMutationBinding<S>,
    {
        let mut binding = Self::for_mutation_identities(identities);
        binding.authored_stage = Some(*identities.edit_stage_identity()?);
        Some(binding)
    }

    pub(in crate::domain_computation::primary_graph) const fn authored_stage_identity(
        &self,
    ) -> Option<&worth_query_declaration::facade::application_operation::ApplicationEditStageIdentity>
    {
        self.authored_stage.as_ref()
    }

    /// Names `Binding` as the mutation this request runs.
    pub(in crate::domain_computation::primary_graph) fn bind_mutation<Schema, Binding>(
        mut self,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.mutation_binding_identity = Some(slot(Binding::IDENTITY));
        self
    }

    /// Whether the slot names exactly this mutation binding identity.
    pub fn is_for_mutation_binding(&self, binding_identity: &str) -> bool {
        self.mutation_binding_identity == Some(slot(binding_identity))
    }
}

pub(super) fn append_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "mutation-binding", identity);
    }
}

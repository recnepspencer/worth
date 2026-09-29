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
    pub(in crate::domain_computation::primary_graph) fn is_for_mutation_binding(
        &self,
        binding_identity: &str,
    ) -> bool {
        self.mutation_binding_identity == Some(slot(binding_identity))
    }
}

pub(super) fn append_identity_slot(encoded: &mut String, identity: Option<[u8; 32]>) {
    if identity.is_some() {
        append_optional_identity_slot(encoded, "mutation-binding", identity);
    }
}

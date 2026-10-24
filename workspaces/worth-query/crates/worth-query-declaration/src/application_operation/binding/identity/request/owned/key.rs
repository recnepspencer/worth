//! A canonical client key can select existing evidence before an input is recovered.
use super::*;

/// Owns a binding's client key and its single canonical encoding.
/// The key identity is descriptive; it grants no access to a recorded input.
pub struct ApplicationEncodedMutationKey<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    key: Binding::IdempotencyKey,
    canonical: super::super::super::ApplicationCanonicalIdentity,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema, Binding> ApplicationEncodedMutationKey<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn encode(key: Binding::IdempotencyKey) -> Result<Self, ApplicationMutationIdentityDenial> {
        let canonical = application_mutation_key_identity::<Schema, Binding>(&key)
            .map_err(ApplicationMutationIdentityDenial::Key)?;
        Ok(Self {
            key,
            canonical,
            marker: PhantomData,
        })
    }

    pub fn key(&self) -> &Binding::IdempotencyKey {
        &self.key
    }
    pub fn identity(&self) -> &[u8; 32] {
        &self.canonical.identity
    }

    /// Joins the actual existing input without serializing either value again.
    pub fn with_input(
        self,
        input: &ApplicationEncodedInput<Binding::InputBinding>,
    ) -> ApplicationOwnedMutationIdentities<Schema, Binding> {
        ApplicationOwnedMutationIdentities {
            key: self.key,
            input: OwnedMutationInput::Shared(input.shared_input().clone()),
            key_identity: self.canonical.identity(),
            input_identity: *input.identity(),
            work: self.canonical.work().combine(input.canonical_work()),
            capture: input.canonical_input().cloned(),
            marker: PhantomData,
        }
    }
}

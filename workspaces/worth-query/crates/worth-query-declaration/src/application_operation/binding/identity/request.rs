//! Both identities of one mutation request, encoded once beside the request.

use std::marker::PhantomData;

use super::{
    application_mutation_key_identity, input_identity, ApplicationCanonicalWork,
    ApplicationEncodedInput,
};
use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{ApplicationSchema, ApplicationValueEncodeDenial};

/// Which part of a mutation request could not be encoded into its identity.
///
/// The payload keeps the encoder's own denial, which names the input type or
/// the key namespace that rejected the value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationMutationIdentityDenial {
    /// The idempotency key did not encode.
    Key(ApplicationValueEncodeDenial),
    /// The mutation input did not encode.
    Input(ApplicationValueEncodeDenial),
}

/// One request to `Binding`: its client key and input together with the
/// identities encoded from exactly those two values.
///
/// Only [`encode`](Self::encode) makes one, so the identities always describe
/// the key and input they travel with. Whatever decides on the input, from the
/// handler to the idempotency binding, reads that input and those identities
/// from this one value; there is no way to pair the identities of one request
/// with the input of another. The binding type travels with them, so they
/// cannot be applied to another binding's request either.
pub struct ApplicationMutationIdentities<'request, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    key: &'request Binding::IdempotencyKey,
    input: &'request Binding::Input,
    key_identity: [u8; 32],
    input_identity: [u8; 32],
    work: ApplicationCanonicalWork,
    marker: PhantomData<fn() -> Schema>,
}

// Derives would demand `Schema` and `Binding` implement the same traits; the
// fields hold only shared references and digests, so these hold for every pair.
impl<Schema, Binding> Clone for ApplicationMutationIdentities<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Binding> Copy for ApplicationMutationIdentities<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
}

impl<Schema, Binding> std::fmt::Debug for ApplicationMutationIdentities<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationMutationIdentities")
            .field("key_identity", &self.key_identity)
            .field("input_identity", &self.input_identity)
            .finish_non_exhaustive()
    }
}

impl<Schema, Binding> PartialEq for ApplicationMutationIdentities<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_identity == other.key_identity && self.input_identity == other.input_identity
    }
}

impl<Schema, Binding> Eq for ApplicationMutationIdentities<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
}

impl<'request, Schema, Binding> ApplicationMutationIdentities<'request, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Encodes the key first, then the input, and reports the part that failed.
    pub fn encode(
        key: &'request Binding::IdempotencyKey,
        input: &'request Binding::Input,
    ) -> Result<Self, ApplicationMutationIdentityDenial> {
        let key_identity = application_mutation_key_identity::<Schema, Binding>(key)
            .map_err(ApplicationMutationIdentityDenial::Key)?;
        let input_identity = input_identity::<Binding::InputBinding>(input)
            .map_err(ApplicationMutationIdentityDenial::Input)?;
        Ok(Self {
            key,
            input,
            key_identity: key_identity.identity(),
            input_identity: input_identity.identity(),
            work: key_identity.work().combine(input_identity.work()),
            marker: PhantomData,
        })
    }

    /// Encodes only the key, beside an input the request already encoded.
    ///
    /// The input and its identity come from `input`, and so does the input's
    /// share of the work: `canonical_work` reports the key's derivation and
    /// whatever work `input` still had to report, so an input encoded when a
    /// request bound its workflow requirement is counted once, here.
    pub fn with_encoded_input(
        key: &'request Binding::IdempotencyKey,
        input: &'request ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, ApplicationMutationIdentityDenial> {
        let key_identity = application_mutation_key_identity::<Schema, Binding>(key)
            .map_err(ApplicationMutationIdentityDenial::Key)?;
        Ok(Self {
            key,
            input: input.input(),
            key_identity: key_identity.identity(),
            input_identity: *input.identity(),
            work: key_identity.work().combine(input.canonical_work()),
            marker: PhantomData,
        })
    }

    /// The input and its identity as one carrier, for an admission that
    /// governs this request's input without encoding it again.
    ///
    /// The carrier reports no work of its own: `canonical_work` here already
    /// reports the input's derivation.
    pub fn encoded_input(&self) -> ApplicationEncodedInput<Binding::InputBinding>
    where
        Binding::Input: Clone,
    {
        ApplicationEncodedInput::reported_elsewhere(self.input.clone(), self.input_identity)
    }

    /// The client idempotency key these identities were encoded from.
    pub const fn idempotency_key(&self) -> &'request Binding::IdempotencyKey {
        self.key
    }

    /// The mutation input these identities were encoded from.
    pub const fn mutation_input(&self) -> &'request Binding::Input {
        self.input
    }

    /// Identity of the client idempotency key, scoped to the binding's key namespace.
    pub const fn key_identity(&self) -> &[u8; 32] {
        &self.key_identity
    }

    /// Identity of the mutation input, scoped to its input type.
    pub const fn input_identity(&self) -> &[u8; 32] {
        &self.input_identity
    }

    /// The work that derived both identities: one derivation for the key and
    /// one for the input, each counted once, whenever the request encoded it.
    pub const fn canonical_work(&self) -> ApplicationCanonicalWork {
        self.work
    }
}

//! Own the exact encoded request while its owner waits for prepared computation.
use super::super::ApplicationSharedInput;
use super::*;
mod key;
pub use key::ApplicationEncodedMutationKey;

/// Key, input and their encoded identities retained for owner completion.
/// Borrowing lends the existing request view without cloning values or encoding
/// either identity again. No constructor accepts descriptive digests.
pub struct ApplicationOwnedMutationIdentities<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    key: Binding::IdempotencyKey,
    input: OwnedMutationInput<Binding::Input>,
    key_identity: [u8; 32],
    input_identity: [u8; 32],
    work: ApplicationCanonicalWork,
    capture: Option<ApplicationCanonicalInput>,
    marker: PhantomData<fn() -> Schema>,
}

enum OwnedMutationInput<Input> {
    Value(Input),
    Shared(ApplicationSharedInput<Input>),
}

impl<Schema, Binding> ApplicationOwnedMutationIdentities<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn encode(
        key: Binding::IdempotencyKey,
        input: Binding::Input,
    ) -> Result<Self, ApplicationMutationIdentityDenial> {
        let encoded = ApplicationMutationIdentities::<Schema, Binding>::encode(&key, &input)?;
        let key_identity = encoded.key_identity;
        let input_identity = encoded.input_identity;
        let work = encoded.work;
        let capture = encoded.capture;
        Ok(Self {
            key,
            input: OwnedMutationInput::Value(input),
            key_identity,
            input_identity,
            work,
            capture,
            marker: PhantomData,
        })
    }

    /// Encodes once under the caller's current canonical work admission.
    pub fn encode_admitted<F, E>(
        key: Binding::IdempotencyKey,
        input: Binding::Input,
        admission: &mut F,
    ) -> Result<Self, ApplicationMutationIdentityAdmittedDenial<E>>
    where
        F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
        E: Debug,
    {
        let encoded = ApplicationMutationIdentities::<Schema, Binding>::encode_admitted(
            &key, &input, admission,
        )?;
        let (key_identity, input_identity, work, capture) = (
            encoded.key_identity,
            encoded.input_identity,
            encoded.work,
            encoded.capture,
        );
        Ok(Self {
            key,
            input: OwnedMutationInput::Value(input),
            key_identity,
            input_identity,
            work,
            capture,
            marker: PhantomData,
        })
    }

    /// Owns the key while sharing the original resource-owned encoded input.
    pub fn with_encoded_input(
        key: Binding::IdempotencyKey,
        input: &ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, ApplicationMutationIdentityDenial> {
        Ok(ApplicationEncodedMutationKey::<Schema, Binding>::encode(key)?.with_input(input))
    }

    pub fn borrow(&self) -> ApplicationMutationIdentities<'_, Schema, Binding> {
        let Self {
            key,
            input,
            key_identity,
            input_identity,
            work,
            capture,
            marker: PhantomData,
        } = self;
        ApplicationMutationIdentities {
            key,
            input: match input {
                OwnedMutationInput::Value(input) => MutationInput::Borrowed(input),
                OwnedMutationInput::Shared(input) => MutationInput::Shared(input.clone()),
            },
            key_identity: *key_identity,
            input_identity: *input_identity,
            work: *work,
            capture: capture.clone(),
            marker: PhantomData,
        }
    }

    pub fn key(&self) -> &Binding::IdempotencyKey {
        &self.key
    }

    /// Includes additional canonical command framing performed by the native entry.
    pub fn with_additional_canonical_work(mut self, work: ApplicationCanonicalWork) -> Self {
        self.work = self.work.combine(work);
        self
    }
}

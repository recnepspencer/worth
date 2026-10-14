//! Own the exact encoded request while its owner waits for prepared computation.
use super::*;

/// Key, input and their encoded identities retained for owner completion.
/// Borrowing lends the existing request view without cloning values or encoding
/// either identity again. No constructor accepts descriptive digests.
pub struct ApplicationOwnedMutationIdentities<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    key: Binding::IdempotencyKey,
    input: Binding::Input,
    key_identity: [u8; 32],
    input_identity: [u8; 32],
    work: ApplicationCanonicalWork,
    capture: Option<ApplicationCanonicalInput>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema, Binding> ApplicationOwnedMutationIdentities<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
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
        let key_identity = encoded.key_identity;
        let input_identity = encoded.input_identity;
        let work = encoded.work;
        let capture = encoded.capture;
        Ok(Self {
            key,
            input,
            key_identity,
            input_identity,
            work,
            capture,
            marker: PhantomData,
        })
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
            input,
            key_identity: *key_identity,
            input_identity: *input_identity,
            work: *work,
            capture: capture.clone(),
            marker: PhantomData,
        }
    }
}

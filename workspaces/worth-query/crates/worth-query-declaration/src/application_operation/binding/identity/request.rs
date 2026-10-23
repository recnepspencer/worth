//! Both identities of one mutation request, encoded once beside the request.

mod owned;
pub use owned::{ApplicationEncodedMutationKey, ApplicationOwnedMutationIdentities};

use std::{fmt::Debug, marker::PhantomData};

use super::{
    application_mutation_key_identity, application_mutation_key_identity_admitted,
    encoder::CanonicalEncodeError, input_identity, input_identity_admitted,
    ApplicationCanonicalInput, ApplicationCanonicalWork, ApplicationEncodedInput,
    CanonicalEncodingCharge,
};
use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{ApplicationSchema, ApplicationValueEncodeDenial};

enum MutationInput<'request, Input> {
    Borrowed(&'request Input),
    Shared(super::ApplicationSharedInput<Input>),
}

impl<Input> Clone for MutationInput<'_, Input> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(input) => Self::Borrowed(input),
            Self::Shared(input) => Self::Shared(input.clone()),
        }
    }
}

impl<Input> AsRef<Input> for MutationInput<'_, Input> {
    fn as_ref(&self) -> &Input {
        match self {
            Self::Borrowed(input) => input,
            Self::Shared(input) => input.as_ref(),
        }
    }
}

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

/// A refusal from the admitted encoder, distinct from a value's serde refusal.
#[derive(Debug)]
pub enum ApplicationMutationIdentityAdmittedDenial<E: Debug> {
    Key(ApplicationValueEncodeDenial),
    Input(ApplicationValueEncodeDenial),
    Admission(E),
    CapacityOverflow,
    Allocation,
}

pub(super) fn admitted_denial<E: Debug>(
    error: CanonicalEncodeError<E>,
    serialization: ApplicationValueEncodeDenial,
    key: bool,
) -> ApplicationMutationIdentityAdmittedDenial<E> {
    match error {
        CanonicalEncodeError::Serialization if key => {
            ApplicationMutationIdentityAdmittedDenial::Key(serialization)
        }
        CanonicalEncodeError::Serialization => {
            ApplicationMutationIdentityAdmittedDenial::Input(serialization)
        }
        CanonicalEncodeError::Admission(error) => {
            ApplicationMutationIdentityAdmittedDenial::Admission(error)
        }
        CanonicalEncodeError::AdmissionDeferred => {
            unreachable!("the canonical owner returns its latched admission cause")
        }
        CanonicalEncodeError::CapacityOverflow => {
            ApplicationMutationIdentityAdmittedDenial::CapacityOverflow
        }
        CanonicalEncodeError::Allocation => ApplicationMutationIdentityAdmittedDenial::Allocation,
    }
}

/// One request to `Binding`: its client key and input together with the
/// identities encoded from exactly those two values.
///
/// Encoding derives the identities from the request; [`with_encoded_input`](Self::with_encoded_input)
/// instead shares the payload and identity of an already encoded input. Whatever decides on the input, from the
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
    input: MutationInput<'request, Binding::Input>,
    key_identity: [u8; 32],
    input_identity: [u8; 32],
    work: ApplicationCanonicalWork,
    capture: Option<ApplicationCanonicalInput>,
    marker: PhantomData<fn() -> Schema>,
}

// Derives would demand `Schema` and `Binding` implement the same traits; the
// The optional snapshot shares its native captured backing without re-encoding.
impl<Schema, Binding> Clone for ApplicationMutationIdentities<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            input: self.input.clone(),
            key_identity: self.key_identity,
            input_identity: self.input_identity,
            work: self.work,
            capture: self.capture.clone(),
            marker: PhantomData,
        }
    }
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
            input: MutationInput::Borrowed(input),
            key_identity: key_identity.identity(),
            input_identity: input_identity.identity(),
            work: key_identity.work().combine(input_identity.work()),
            capture: input_identity.capture,
            marker: PhantomData,
        })
    }

    /// Encodes both identities under one cumulative work and scratch admission.
    /// Each charge is requested before the corresponding hash, copy or growth.
    pub fn encode_admitted<F, E>(
        key: &'request Binding::IdempotencyKey,
        input: &'request Binding::Input,
        admission: &mut F,
    ) -> Result<Self, ApplicationMutationIdentityAdmittedDenial<E>>
    where
        F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
        E: Debug,
    {
        let key_identity =
            application_mutation_key_identity_admitted::<Schema, Binding, _, _>(key, admission)
                .map_err(|error| {
                    admitted_denial(
                        error,
                        super::rejected(
                            crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                                Binding::IDEMPOTENCY_IDENTITY,
                            ),
                        ),
                        true,
                    )
                })?;
        let input_identity =
            input_identity_admitted::<Binding::InputBinding, _, _>(input, admission)?;
        Ok(Self {
            key,
            input: MutationInput::Borrowed(input),
            key_identity: key_identity.identity(),
            input_identity: input_identity.identity(),
            work: key_identity.work().combine(input_identity.work()),
            capture: input_identity.capture,
            marker: PhantomData,
        })
    }

    /// Encodes only the key, beside an input the request already encoded.
    ///
    /// The input and its identity come from `input`, and so does the input's
    /// share of the work: `canonical_work` reports the key's derivation and
    /// whatever work `input` still had to report, so an input encoded when a
    /// request bound its workflow requirement is counted once, here.
    /// The shared payload remains owned after the encoded wrapper is dropped.
    pub fn with_encoded_input(
        key: &'request Binding::IdempotencyKey,
        input: &ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Result<Self, ApplicationMutationIdentityDenial> {
        let key_identity = application_mutation_key_identity::<Schema, Binding>(key)
            .map_err(ApplicationMutationIdentityDenial::Key)?;
        Ok(Self {
            key,
            input: MutationInput::Shared(input.shared_input().clone()),
            key_identity: key_identity.identity(),
            input_identity: *input.identity(),
            work: key_identity.work().combine(input.canonical_work()),
            capture: input.canonical_input().cloned(),
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
        match &self.input {
            MutationInput::Borrowed(input) => ApplicationEncodedInput::reported_elsewhere(
                (*input).clone(),
                self.input_identity,
                self.capture.clone(),
            ),
            MutationInput::Shared(input) => ApplicationEncodedInput::reported_shared_elsewhere(
                input.clone(),
                self.input_identity,
                self.capture.clone(),
            ),
        }
    }

    /// The client idempotency key these identities were encoded from.
    pub const fn idempotency_key(&self) -> &'request Binding::IdempotencyKey {
        self.key
    }

    /// The mutation input these identities were encoded from.
    pub fn mutation_input(&self) -> &Binding::Input {
        self.input.as_ref()
    }

    /// Identity of the client idempotency key, scoped to the binding's key namespace.
    pub const fn key_identity(&self) -> &[u8; 32] {
        &self.key_identity
    }

    /// Identity of the mutation input, scoped to its input type.
    pub const fn input_identity(&self) -> &[u8; 32] {
        &self.input_identity
    }

    pub const fn canonical_input(&self) -> Option<&ApplicationCanonicalInput> {
        self.capture.as_ref()
    }

    /// The work that derived both identities: one derivation for the key and
    /// one for the input, each counted once, whenever the request encoded it.
    pub const fn canonical_work(&self) -> ApplicationCanonicalWork {
        self.work
    }
}

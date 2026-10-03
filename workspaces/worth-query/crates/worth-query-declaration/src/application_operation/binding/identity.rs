//! Operation input and idempotency-key identities derived from canonical encoding.
//!
//! Authors never write these hashes. Everything a `Serialize` value emits takes
//! part, so adding a derived field cannot leave it out of the identity, and the
//! prefix-free encoding keeps different serialized values from sharing one
//! identity. A retry that reuses a key with a changed input is therefore
//! refused as intent drift instead of replaying the earlier result.
//!
//! The identity is only as complete as the serialization. A skipped field
//! (`#[serde(skip)]`, `skip_serializing_if`) or a lossy custom `Serialize` does
//! not take part, and neither does anything serde never sees: `PhantomData`
//! and generic type parameters emit no data, so a `Money<C>` currency marker is
//! not in the identity. Put anything that must matter in a serialized field.
//! Maps are sorted by their encoded keys, but a `HashSet` or any other
//! sequence encodes in iteration order.
//! Serialized type, field and variant names are part of the identity, but
//! `#[serde(untagged)]` emits no variant name, so variants with the same payload
//! share one identity. Integers encode by value, so a `u8` and a `u64` holding
//! the same number agree, but a signed and an unsigned integer holding the same
//! number differ; the input type in the scope keeps bindings with different
//! input types apart.
//! Floats encode their exact bits, so `-0.0` and `0.0` differ.
//!
//! The input identity is scoped to the input type, not to the binding, so a
//! workflow proposal and the guarded operation it names agree on the input they
//! carry. Every request encodes its input exactly once and its key exactly
//! once. A mutation request encodes both at its entry point into
//! `ApplicationMutationIdentities`, which keeps both values beside the
//! identities encoded from them and the deterministic work (`canonical_work`)
//! that encoding took. A request bound to a workflow requirement encodes its
//! input when it binds, into an [`ApplicationEncodedInput`], and its entry
//! point then encodes only the key beside it. Capability admission of a
//! mutation request takes the input identity from those identities, and a
//! capability workflow request encodes its input into an
//! `ApplicationEncodedInput` that its admission and its idempotency binding
//! both reuse. Admission, the handler and the commit never encode the input
//! again, and the request's derivation work is reported once, in the admission
//! phase of the receipt's canonical-work evidence, on fresh commits and
//! replayed retries alike. The binding itself joins the idempotency intent
//! through `WorthQueryApplicationIdempotencyBinding::for_mutation_identities`,
//! which keeps two bindings of one operation from replaying each other under
//! one key; a commit the host authors itself uses `for_host_commit`, scoped to
//! the operation it commits under, and is the only constructor a host names.

mod encoder;
mod input;
mod request;
mod work;

pub use encoder::CanonicalEncodingCharge;
pub use input::ApplicationEncodedInput;
pub use request::{
    ApplicationMutationIdentities, ApplicationMutationIdentityAdmittedDenial,
    ApplicationMutationIdentityDenial,
};
pub use work::ApplicationCanonicalWork;

use serde::Serialize;

use encoder::Sink;

use super::ApplicationMutationBinding;
use crate::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding, ApplicationValueEncodeDenial,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

const INPUT_DOMAIN: &str = "worth-query.operation-input.v1";
const KEY_DOMAIN: &str = "worth-query.idempotency-key.v1";
const REJECTED: &str = "value serialization rejected";

/// What a canonical value identity is for.
///
/// Each purpose encodes under its own domain, so two purposes never share an
/// identity for equal values. These are the purposes outside mutation
/// bindings, whose key and input identities come from
/// [`ApplicationMutationIdentities`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ApplicationValueIdentityDomain {
    /// The client key of a capability workflow request together with the
    /// principal that sent it, scoped to the workflow's operation.
    CapabilityWorkflowKey,
    /// Everything that makes one temporal intent wake the same intent.
    TemporalIntentRelation,
    /// The client key of a commit whose effects the host authored itself.
    HostCommitKey,
    /// The intent of a commit whose effects the host authored itself.
    HostCommitIntent,
    /// The installed semantic meaning of one output producer binding.
    ProducerImplementationEdition,
}

impl ApplicationValueIdentityDomain {
    const fn text(self) -> &'static str {
        match self {
            Self::CapabilityWorkflowKey => "worth-query.capability-workflow-key.v1",
            Self::TemporalIntentRelation => "worth-query.temporal-intent-idempotency-relation.v1",
            Self::HostCommitKey => "worth-query.host-commit-key.v1",
            Self::HostCommitIntent => "worth-query.host-commit-intent.v1",
            Self::ProducerImplementationEdition => "worth-query.producer-implementation-edition.v1",
        }
    }
}

/// A canonical identity together with the work that derived it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationCanonicalIdentity {
    identity: [u8; 32],
    work: ApplicationCanonicalWork,
}

impl ApplicationCanonicalIdentity {
    /// The 32-byte identity.
    pub const fn identity(self) -> [u8; 32] {
        self.identity
    }

    /// The deterministic work that derived it.
    pub const fn work(self) -> ApplicationCanonicalWork {
        self.work
    }
}

/// Identity of one operation input, scoped to its input type.
///
/// Only [`ApplicationEncodedInput::encode`] and
/// [`ApplicationMutationIdentities::encode`] call it, so each keeps the
/// identity beside the value it was encoded from.
fn input_identity<InputBinding>(
    input: &InputBinding::Value,
) -> Result<ApplicationCanonicalIdentity, ApplicationValueEncodeDenial>
where
    InputBinding: ApplicationStructuredValueBinding,
    InputBinding::Value: Serialize,
{
    let input_type = InputBinding::IDENTITY;
    canonical_identity(INPUT_DOMAIN, input_type.as_str(), input).map_err(|_| rejected(input_type))
}

fn input_identity_admitted<InputBinding, F, E>(
    input: &InputBinding::Value,
    admission: &mut F,
) -> Result<ApplicationCanonicalIdentity, encoder::CanonicalEncodeError<E>>
where
    InputBinding: ApplicationStructuredValueBinding,
    InputBinding::Value: Serialize,
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: std::fmt::Debug,
{
    canonical_identity_admitted(
        INPUT_DOMAIN,
        InputBinding::IDENTITY.as_str(),
        input,
        admission,
    )
}

/// Identity of one client idempotency key, scoped to the binding's key namespace.
fn application_mutation_key_identity<Schema, Binding>(
    key: &Binding::IdempotencyKey,
) -> Result<ApplicationCanonicalIdentity, ApplicationValueEncodeDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    canonical_identity(KEY_DOMAIN, Binding::IDEMPOTENCY_IDENTITY, key).map_err(|_| {
        rejected(WorthQueryPortableTypeIdentity::declared(
            Binding::IDEMPOTENCY_IDENTITY,
        ))
    })
}

fn application_mutation_key_identity_admitted<Schema, Binding, F, E>(
    key: &Binding::IdempotencyKey,
    admission: &mut F,
) -> Result<ApplicationCanonicalIdentity, encoder::CanonicalEncodeError<E>>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: std::fmt::Debug,
{
    canonical_identity_admitted(KEY_DOMAIN, Binding::IDEMPOTENCY_IDENTITY, key, admission)
}

/// Identity of any value for one purpose inside a scope.
///
/// This is the encoding entry for identities that are neither an operation
/// input nor a mutation binding's key, such as the key of a capability
/// workflow request. The contract matches
/// the binding identities: `value` is encoded canonically from its `Serialize`
/// output, `domain` names what the identity is for and `scope` narrows it, and
/// the identity changes with either. A value that cannot serialize fails with
/// `CodecRejected` naming the domain, which is reported and never part of the
/// hash.
pub fn application_value_identity<T: Serialize + ?Sized>(
    domain: ApplicationValueIdentityDomain,
    scope: &str,
    value: &T,
) -> Result<ApplicationCanonicalIdentity, ApplicationValueEncodeDenial> {
    canonical_identity(domain.text(), scope, value)
        .map_err(|_| rejected(WorthQueryPortableTypeIdentity::declared(domain.text())))
}

fn canonical_identity<T: Serialize + ?Sized>(
    domain: &str,
    scope: &str,
    value: &T,
) -> Result<ApplicationCanonicalIdentity, encoder::CanonicalEncodeError> {
    canonical_identity_in_sink(domain, scope, value, encoder::HashingSink::new())
}

fn canonical_identity_admitted<T: Serialize + ?Sized, F, E>(
    domain: &str,
    scope: &str,
    value: &T,
    admission: &mut F,
) -> Result<ApplicationCanonicalIdentity, encoder::CanonicalEncodeError<E>>
where
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: std::fmt::Debug,
{
    canonical_identity_in_sink(
        domain,
        scope,
        value,
        encoder::HashingSink::with_admission(encoder::CallbackAdmission::new(admission)),
    )
}

fn canonical_identity_in_sink<T: Serialize + ?Sized, A: encoder::EncodingAdmission>(
    domain: &str,
    scope: &str,
    value: &T,
    mut sink: encoder::HashingSink<A>,
) -> Result<ApplicationCanonicalIdentity, encoder::CanonicalEncodeError<A::Error>> {
    let encoded = (|| {
        let mut framing = 0_usize;
        for part in [domain, scope] {
            let length = (part.len() as u64).to_be_bytes();
            sink.put(&length)?;
            sink.put(part.as_bytes())?;
            framing = framing.saturating_add(length.len() + part.len());
        }
        encoder::encode_into(&mut sink, value)?;
        Ok::<_, encoder::CanonicalEncodeError<A::Error>>(framing)
    })();
    if let Some(error) = sink.take_terminal_failure() {
        return Err(error);
    }
    let framing = encoded?;
    let hashed = sink.hashed_bytes();
    let work = ApplicationCanonicalWork::one_derivation(
        hashed.saturating_sub(framing),
        hashed,
        sink.buffered_bytes(),
    );
    Ok(ApplicationCanonicalIdentity {
        identity: sink.finish()?,
        work,
    })
}

fn rejected(binding_identity: WorthQueryPortableTypeIdentity) -> ApplicationValueEncodeDenial {
    ApplicationValueEncodeDenial::CodecRejected {
        binding_identity,
        reason: REJECTED,
    }
}

#[cfg(test)]
mod tests;

//! One operation input together with the identity encoded from it.

use serde::Serialize;

use super::{input_identity, ApplicationCanonicalWork};
use crate::application_schema::{ApplicationStructuredValueBinding, ApplicationValueEncodeDenial};

/// An operation input and its canonical identity, scoped to `InputBinding`.
///
/// Only [`encode`](Self::encode) derives one, and
/// [`ApplicationMutationIdentities::encoded_input`](super::ApplicationMutationIdentities::encoded_input)
/// copies one out of a request that already encoded its input, so the identity
/// always describes the value it travels with and the input type in the scope
/// is the carrier's own. Whatever needs the input identity after the request
/// entry, from a capability admission to a workflow requirement, takes it from
/// here instead of encoding the input again.
///
/// `canonical_work` is the derivation work this carrier still has to report:
/// the encoding it performed, or none when the request's
/// `ApplicationMutationIdentities` already report it. Input custody is immutable and shared.
/// A clone carries the same
/// work, because only one of two admissions of one request ever reaches a
/// receipt.
pub struct ApplicationEncodedInput<InputBinding>
where
    InputBinding: ApplicationStructuredValueBinding,
{
    input: std::sync::Arc<InputBinding::Value>,
    identity: [u8; 32],
    work: ApplicationCanonicalWork,
}

impl<InputBinding> ApplicationEncodedInput<InputBinding>
where
    InputBinding: ApplicationStructuredValueBinding,
    InputBinding::Value: Serialize,
{
    /// Encodes `input` once into its identity, scoped to its input type.
    ///
    /// This is the only derivation of an input identity: every binding and
    /// every capability that admits the same input type derives the same
    /// identity for the same value, so a workflow proposal, the guarded
    /// operation it names and the capability admission of that operation's
    /// input all agree on the input they carry.
    pub fn encode(input: InputBinding::Value) -> Result<Self, ApplicationValueEncodeDenial> {
        let derived = input_identity::<InputBinding>(&input)?;
        Ok(Self {
            input: std::sync::Arc::new(input),
            identity: derived.identity(),
            work: derived.work(),
        })
    }
}

impl<InputBinding> ApplicationEncodedInput<InputBinding>
where
    InputBinding: ApplicationStructuredValueBinding,
{
    pub(super) fn reported_elsewhere(input: InputBinding::Value, identity: [u8; 32]) -> Self {
        Self {
            input: std::sync::Arc::new(input),
            identity,
            work: ApplicationCanonicalWork::none(),
        }
    }

    /// The input the identity was encoded from.
    pub fn input(&self) -> &InputBinding::Value {
        self.input.as_ref()
    }

    /// Identity of the input, scoped to its input type.
    pub const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    /// The derivation work this carrier still has to report.
    pub const fn canonical_work(&self) -> ApplicationCanonicalWork {
        self.work
    }

    /// Gives native admission shared immutable custody of the encoded input.
    pub fn into_shared_input(self) -> std::sync::Arc<InputBinding::Value> {
        self.input
    }
}

impl<InputBinding> Clone for ApplicationEncodedInput<InputBinding>
where
    InputBinding: ApplicationStructuredValueBinding,
{
    fn clone(&self) -> Self {
        Self {
            input: std::sync::Arc::clone(&self.input),
            identity: self.identity,
            work: self.work,
        }
    }
}

impl<InputBinding> std::fmt::Debug for ApplicationEncodedInput<InputBinding>
where
    InputBinding: ApplicationStructuredValueBinding,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationEncodedInput")
            .field("identity", &self.identity)
            .field("work", &self.work)
            .finish_non_exhaustive()
    }
}

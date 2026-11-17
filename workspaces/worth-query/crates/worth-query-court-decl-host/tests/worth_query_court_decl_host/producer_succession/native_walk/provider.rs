//! Keep the fixture's realization topology fixed while real source Width changes.
use super::*;
use producers::parent::{self, preserve};
pub struct Producer;
pub struct Provider;
impl WorthQueryApplicationProducerBinding<Schema> for Producer {
    type Operation = preserve::Binding;
    type OutputFamily = parent::Family;
    type Provider = Provider;
    type OutputRole = preserve::Output;
    const IDENTITY: &'static str =
        <preserve::Producer as WorthQueryApplicationProducerBinding<Schema>>::IDENTITY;
    const APPLICABILITY: &'static [WorthQueryProducerApplicability] =
        <preserve::Producer as WorthQueryApplicationProducerBinding<Schema>>::APPLICABILITY;
    const REQUIRED_INVARIANTS: &'static [WorthQueryProducerInvariantRequirement] = &[];
    const OWNERSHIP: ProducerPublicationOwnership =
        <preserve::Producer as WorthQueryApplicationProducerBinding<Schema>>::OWNERSHIP;
    const REUSE_POLICY: &'static str = "walk-only-fixed-realization";
}
impl WorthQueryApplicationProducerProvider<Schema, Producer> for Provider {
    type DenialValue = WorthQueryUndeclaredProducerDenial;
    const SEMANTIC_IDENTITY: &'static str = "walk-producer.fixed-realization";
    fn operation_input(&self, row: &Row) -> preserve::Input {
        let mut input = row.clone();
        input.width = 0;
        preserve::Input(input)
    }
    fn idempotency_key(&self, _: &Row, identity: &[u8; 32]) -> u64 {
        u64::from_le_bytes(identity[..8].try_into().unwrap())
    }
    fn domain_denial_reason(&self, denial: &Denial) -> Option<&'static str> {
        <preserve::Provider as WorthQueryApplicationProducerProvider<Schema,preserve::Producer>>::domain_denial_reason(&preserve::Provider,denial)
    }
}

// Readiness binds the actual installed producer type and its declared meaning.
type Operation = preserve::Operation;
type Input = preserve::Input;
use realization::RealizedValue;
readiness!(
    "dispatch.split.parent.preserve",
    RealizedValue,
    realization::canonical_query,
    "realization"
);

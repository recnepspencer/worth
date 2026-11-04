//! The rejecting producer's declared domain value, beside its diagnostic text.
use super::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    WorthQueryProducerDenialDecodeDenial, WorthQueryProducerDenialValue,
};
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;

/// A producer's domain decision: its diagnostic text and declared value travel
/// together in one box, so the demand denial stays as small as one without it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProducerDomainDenial {
    pub(super) reason: Option<&'static str>,
    pub(super) value: WorthQueryProducerDenialValue,
}

impl WorthQueryOutputDemandDenial {
    /// The value the rejecting producer declared for its domain denial. `None`
    /// exactly when the demand was not refused by a producer's domain decision.
    /// Diagnostic clones retain the same value.
    pub fn producer_denial_value(&self) -> Option<&WorthQueryProducerDenialValue> {
        self.domain.as_ref().map(|domain| &domain.value)
    }

    /// Decode the producer's domain denial with its declared binding.
    pub fn decode_producer_denial<Binding: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<Binding::Value, WorthQueryProducerDenialDecodeDenial> {
        self.producer_denial_value()
            .ok_or(WorthQueryProducerDenialDecodeDenial::NotDomainDenied)?
            .decode::<Binding>()
    }

    /// The exact canonical bytes of the producer's declared domain value.
    pub fn producer_denial_bytes(&self) -> Option<&[u8]> {
        self.producer_denial_value()?.bytes()
    }

    pub(super) fn producer_denial_value_charged_bytes(&self) -> u64 {
        self.domain.as_deref().map_or(0, |domain| {
            (std::mem::size_of_val(domain) as u64)
                .saturating_add(domain.value.owned_capacity_bytes())
        })
    }
}

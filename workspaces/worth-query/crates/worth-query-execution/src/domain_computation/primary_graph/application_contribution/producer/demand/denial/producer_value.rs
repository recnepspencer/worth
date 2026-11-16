//! The rejecting producer's declared domain value, beside its diagnostic text.
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::application_contribution::producer::{
    WorthQueryProducerDenialDecodeDenial, WorthQueryProducerDenialValue,
};
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;

/// A producer's domain decision: its diagnostic text and declared value travel
/// together in one box, so the demand denial stays as small as one without it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProducerDomainDenial {
    pub(super) reason: Option<std::borrow::Cow<'static, str>>,
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
                .saturating_add(match &domain.reason {
                    Some(std::borrow::Cow::Owned(reason)) => reason.capacity() as u64,
                    _ => 0,
                })
        })
    }
}

impl WorthQueryOutputDemandDenial {
    /// Domain-owned diagnostic words, when the rejected producer supplies them.
    /// These words do not replace the typed denial kind or confer authority.
    pub fn domain_reason(&self) -> Option<&str> {
        self.domain
            .as_ref()
            .and_then(|domain| domain.reason.as_deref())
    }
    pub(in crate::domain_computation::primary_graph) fn producer_domain_denied(
        subject: impl Into<std::borrow::Cow<'static, str>>,
        reason: Option<&'static str>,
        value: crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerDenialValue,
    ) -> Self {
        let mut denial = Self::new(
            WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
            subject,
        );
        denial.domain = Some(Box::new(ProducerDomainDenial {
            reason: reason.map(Into::into),
            value,
        }));
        denial
    }
}

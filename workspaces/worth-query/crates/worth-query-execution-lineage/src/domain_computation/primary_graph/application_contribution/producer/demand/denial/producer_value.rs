//! The rejecting producer's declared domain value, beside its diagnostic text.
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::application_contribution::producer::{
    WorthQueryProducerDenialDecodeDenial, WorthQueryProducerDenialValue,
};
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;

/// A producer's domain decision: its diagnostic text and declared value travel
/// together in one box, so the demand denial stays as small as one without it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerDomainDenial {
    pub reason: Option<std::borrow::Cow<'static, str>>,
    pub value: WorthQueryProducerDenialValue,
    /// What the refused decision read (PR C §1.7): a pending refusal is
    /// retried only when one of these proofs changes.
    pub proofs: Option<crate::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::RefusalProofs>,
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

    pub fn producer_denial_value_charged_bytes(&self) -> u64 {
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
    pub fn producer_domain_denied(
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
            proofs: None,
        }));
        denial
    }
    /// A domain refusal keeps what its decision read (PR C §1.7).
    pub fn with_refusal_proofs(mut self, proofs: Option<crate::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::RefusalProofs>) -> Self {
        if let Some(domain) = self.domain.as_mut() {
            domain.proofs = proofs;
        }
        self
    }
    pub fn refusal_proofs(&self) -> Option<&crate::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::RefusalProofs> {
        self.domain.as_ref()?.proofs.as_ref()
    }
}

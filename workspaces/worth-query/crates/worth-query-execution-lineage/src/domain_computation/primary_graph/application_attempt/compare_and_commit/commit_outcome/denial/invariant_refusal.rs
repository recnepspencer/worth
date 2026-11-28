//! Reads domain evidence from the original publication denial.
use super::WorthQueryApplicationCommitDenial;
use crate::domain_computation::primary_graph::application_invariant::{
    refusal, WorthQueryInvariantRefusalDecodeDenial,
};
use crate::domain_computation::primary_graph::{
    WorthQueryDomainRefusal, WorthQueryDomainRefusalBinding,
};
use crate::domain_computation::WorthQueryCustomInvariantDenial;
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_relational::facade::runtime::CustomInvariantRefusal;

impl WorthQueryApplicationCommitDenial {
    /// Decode with the installed rule's declared value binding. A different
    /// binding is refused before its decoder runs.
    pub fn decode_invariant_refusal<Binding: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<Binding::Value, WorthQueryInvariantRefusalDecodeDenial> {
        refusal::decode::<Binding>(self.invariant_refusal()?)
    }

    /// The application's structured reason when the refusing invariant
    /// declares `WorthQueryDomainRefusalBinding` (a violation or an evaluation
    /// `Cause`); `None` for every other refusal, including another binding's.
    pub fn domain_refusal(&self) -> Option<WorthQueryDomainRefusal> {
        self.decode_invariant_refusal::<WorthQueryDomainRefusalBinding>()
            .ok()
    }

    /// The exact canonical bytes evaluated by the refusing invariant.
    pub fn invariant_refusal_bytes(&self) -> Option<&[u8]> {
        match self.invariant_refusal().ok()? {
            CustomInvariantRefusal::Empty => None,
            CustomInvariantRefusal::DeclaredValue { bytes, .. } => Some(bytes),
        }
    }

    fn invariant_refusal(
        &self,
    ) -> Result<&CustomInvariantRefusal, WorthQueryInvariantRefusalDecodeDenial> {
        self.custom_invariant_denial()
            .and_then(WorthQueryCustomInvariantDenial::refusal_payload)
            .ok_or(WorthQueryInvariantRefusalDecodeDenial::NoDomainRefusal)
    }
}

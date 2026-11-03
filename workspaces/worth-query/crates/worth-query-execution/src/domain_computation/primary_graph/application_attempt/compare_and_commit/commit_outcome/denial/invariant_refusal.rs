//! Reads domain evidence from the original publication denial.
use super::WorthQueryApplicationCommitDenial;
use crate::domain_computation::primary_graph::application_invariant::{
    refusal, WorthQueryInvariantRefusalDecodeDenial,
};
use crate::domain_computation::WorthQueryCustomInvariantDenial;
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_relational::facade::runtime::CustomInvariantRefusal;

impl WorthQueryApplicationCommitDenial {
    /// Exact full-generation subject from the installed
    /// ownership invariant. Other invariant refusals have their own codecs.
    pub fn model_ownership_conflict(
        &self,
    ) -> Option<worth_relational::facade::branch::RelationalModelOwnershipConflict> {
        use worth_relational::facade::branch::RelationalModelOwnershipConflict as Conflict;
        match self.invariant_refusal().ok()? {
            CustomInvariantRefusal::DeclaredValue {
                value_identity,
                bytes,
            } if value_identity == Conflict::VALUE_IDENTITY => Conflict::decode(bytes).ok(),
            _ => None,
        }
    }
    /// Decode with the installed rule's declared value binding. A different
    /// binding is refused before its decoder runs.
    pub fn decode_invariant_refusal<Binding: ApplicationReadableScalarValueBinding>(
        &self,
    ) -> Result<Binding::Value, WorthQueryInvariantRefusalDecodeDenial> {
        refusal::decode::<Binding>(self.invariant_refusal()?)
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
        match self.custom_invariant_denial() {
            Some(WorthQueryCustomInvariantDenial::Violation { refusal, .. }) => Ok(refusal),
            Some(WorthQueryCustomInvariantDenial::Failure { .. }) | None => {
                Err(WorthQueryInvariantRefusalDecodeDenial::NoDomainRefusal)
            }
        }
    }
}

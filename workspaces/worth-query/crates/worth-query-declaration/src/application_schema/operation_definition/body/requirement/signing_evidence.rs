//! Consume the producer's completed declaration, not a parallel relation/path mapping.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperands as Operands,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::application_schema::ApplicationSigningEvidenceRef;
impl<Schema, Operation> Operands<Schema, Operation> {
    /// Observes existence, not permission. Multiple matching current rows still mean true.
    pub fn signed_evidence<Binding>(
        self,
        name: &str,
        evidence: &ApplicationSigningEvidenceRef<Schema, Binding>,
    ) -> Result<Self, Denial> {
        self.push(name, Source::SignedEvidence(evidence.source.clone()))
    }
}

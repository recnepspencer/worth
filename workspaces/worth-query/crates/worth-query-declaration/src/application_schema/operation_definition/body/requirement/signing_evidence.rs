//! Consume the producer's completed declaration, not a parallel relation/path mapping.
use super::{
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperands as Operands,
    ApplicationOperationBodyRequirementSource as Source,
};
use crate::application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding};
use crate::application_schema::{
    ApplicationRelationRef, ApplicationSchema, ApplicationSigningEvidenceRef, OperationReads,
};
impl<Schema, Operation> Operands<Schema, Operation> {
    /// Observes existence, not permission. Multiple matching current rows still mean true.
    pub fn signed_evidence<Binding>(
        self,
        name: &str,
        evidence: &ApplicationSigningEvidenceRef<Schema, Binding>,
    ) -> Result<Self, Denial> {
        self.push(name, Source::SignedEvidence(evidence.source.clone()))
    }

    /// Selects one outgoing subject through native graph truth. No target yields false;
    /// the authored expression determines the requirement's status.
    /// multiple distinct targets or a dangling endpoint fail closed. Selection and
    /// evidence observations are sealed together for publication currentness.
    pub fn signed_evidence_from_related<Binding, Relation, Root>(
        self,
        name: &str,
        relation: ApplicationRelationRef<
            Schema,
            Relation,
            Root,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        evidence: &ApplicationSigningEvidenceRef<Schema, Binding>,
    ) -> Result<Self, Denial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Relation: OperationReads<Operation>,
    {
        if relation.to() != evidence.source.lifecycle.source_entity {
            return Err(Denial::ForeignScope);
        }
        self.push(
            name,
            Source::RelatedSignedEvidence {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
                evidence: evidence.source.clone(),
            },
        )
    }
}

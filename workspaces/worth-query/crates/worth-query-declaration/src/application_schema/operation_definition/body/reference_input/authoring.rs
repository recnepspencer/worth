//! Typed source, target identity and relation permissions precede inert binding meaning.
use super::ApplicationOperationBodyReferenceInput as Reference;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationIdentityScalarValueBinding,
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationRelationRef, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    EqualityPredicate, NoApplicationUnit, OperationLinks, OperationReads, OperationUnlinks,
    ReadOnly,
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Replaces a nullable at-most-one outgoing reference with an existing input-selected target.
    /// `Some(key)` resolves one real identity; explicit `None` clears. Missing/malformed input denies.
    /// The target is never created or deleted. Same-endpoint assignment preserves its existing edge.
    pub fn replace_relation_from_input<R, Root, Selected, A, F, V>(
        mut self,
        relation: ApplicationRelationRef<S, R, Root, Selected>,
        identity: ApplicationFieldRef<
            S,
            Selected,
            A,
            F,
            V,
            ReadOnly,
            EqualityPredicate,
            NoApplicationUnit,
        >,
        member: &str,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O>,
        F: OperationReads<O> + DeclaredApplicationFieldValue<Value = V>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        if relation.integrity().cardinality.source_max != Some(1)
            || relation
                .integrity()
                .cardinality
                .source_min
                .is_some_and(|minimum| minimum > 0)
            || F::PRESENCE != crate::application_schema::ApplicationFieldPresence::Required
            || [member, key]
                .iter()
                .any(|value| value.is_empty() || value.trim() != *value)
            || self
                .reference_inputs
                .iter()
                .any(|prior| prior.relation == relation.name())
        {
            return Err(Denial::InvalidReferenceReplacement);
        }
        self.in_named_scope(relation.from())?;
        self.reference_inputs.push(Reference {
            relation: relation.name().into(),
            from: relation.from().into(),
            to: relation.to().into(),
            field: ApplicationFieldBindingLocus::new(
                identity.entity(),
                identity.aspect(),
                identity.field(),
            ),
            binding: F::Binding::IDENTITY,
            scalar_family: F::Binding::SCALAR_FAMILY,
            member: member.into(),
            key: key.into(),
        });
        Ok(self)
    }
}

//! Typed source, target identity and relation permissions precede inert binding meaning.
use super::{
    ApplicationOperationBodyReferenceInput as Reference,
    ApplicationOperationBodyReferenceInputMode as Mode,
};
use crate::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyBuilder, ApplicationRelationRef, ApplicationScalarValueBinding,
    OperationLinks, OperationReads, OperationUnlinks,
};
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Replaces a nullable at-most-one outgoing reference with an existing input-selected target.
    /// `Some(key)` resolves one real identity; explicit `None` clears. Missing/malformed input denies.
    /// The target is never created or deleted. Same-endpoint assignment preserves its existing edge.
    pub fn replace_relation_from_input<R, Root, Selected, F>(
        self,
        relation: ApplicationRelationRef<S, R, Root, Selected>,
        identity: F,
        member: &str,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O>,
        F: crate::application_schema::fragment::RecordSelection<Entity = Selected>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        let cardinality = relation.integrity().cardinality;
        if cardinality.source_max != Some(1) || cardinality.source_min.is_some_and(|min| min > 0) {
            return Err(Denial::InvalidReferenceReplacement);
        }
        self.reference(relation, identity, member, key, Mode::ReplaceNullable)
    }

    /// Links (`linked` is true) or unlinks (false) the exact scope-to-target edge whose
    /// target a required identity `member` selects, for any relation cardinality. An
    /// existing edge is not linked again and an absent edge is not unlinked, so repeats
    /// are no-ops. An unknown target refuses before any effect; neither endpoint is created.
    pub fn toggle_relation_from_input<R, Root, Selected, F>(
        self,
        relation: ApplicationRelationRef<S, R, Root, Selected>,
        identity: F,
        member: &str,
        linked_member: &str,
        key: &str,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Selected: OperationReads<O>,
        R: OperationReads<O> + OperationLinks<O> + OperationUnlinks<O>,
        F: crate::application_schema::fragment::RecordSelection<Entity = Selected>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        let linked_member = linked_member.to_owned();
        if linked_member == member {
            return Err(Denial::InvalidReferenceToggle);
        }
        self.reference(
            relation,
            identity,
            member,
            key,
            Mode::ToggleExact { linked_member },
        )
    }

    fn reference<R, Root, Selected, F>(
        mut self,
        relation: ApplicationRelationRef<S, R, Root, Selected>,
        identity: F,
        member: &str,
        key: &str,
        mode: Mode,
    ) -> Result<Self, Denial>
    where
        F: crate::application_schema::fragment::RecordSelection<Entity = Selected>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        let invalid = match mode {
            Mode::ReplaceNullable => Denial::InvalidReferenceReplacement,
            Mode::ToggleExact { .. } => Denial::InvalidReferenceToggle,
        };
        let identity = super::super::record_authoring::resolve(identity)?;
        let reference = Reference {
            relation: relation.name().into(),
            from: relation.from().into(),
            to: relation.to().into(),
            field: identity.locus().clone(),
            binding: F::Binding::IDENTITY,
            scalar_family: F::Binding::SCALAR_FAMILY,
            member: member.into(),
            key: key.into(),
            mode,
        };
        if identity.writable()
            || !identity.equality_queryable()
            || identity.presence() != crate::application_schema::ApplicationFieldPresence::Required
            || !reference.has_trimmed_names()
            || self
                .reference_inputs
                .iter()
                .any(|prior| prior.relation == reference.relation)
        {
            return Err(invalid);
        }
        self.in_named_scope(relation.from())?;
        self.reference_inputs.push(reference);
        Ok(self)
    }
}

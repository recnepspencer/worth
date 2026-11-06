//! A typed related preimage is captured by Query, never supplied by the caller.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    fragment::RecordSelection, ApplicationFieldBindingLocus, ApplicationFieldPresence,
    ApplicationOperationDecisionReadTarget as Read, ApplicationRelationRef, OperationReads,
};

/// Descriptive outgoing selection and source field; cold admission proves its declared closure.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyRelatedFieldSource {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub field: ApplicationFieldBindingLocus,
    pub presence: ApplicationFieldPresence,
}

#[cfg(test)]
mod tests;
impl ApplicationOperationBodyRelatedFieldSource {
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("related-field-source-v1;");
        for part in [
            self.relation.as_str(),
            self.from.as_str(),
            self.to.as_str(),
            self.field.entity(),
            self.field.aspect(),
            self.field.field(),
            self.presence.canonical_name(),
        ] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
    pub fn selection_read(&self) -> Read {
        Read::Relation {
            relation: self.relation.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
    pub fn decision_reads(&self) -> [Read; 3] {
        [
            self.selection_read(),
            Read::Entity {
                entity: self.from.clone(),
            },
            Read::Entity {
                entity: self.to.clone(),
            },
        ]
    }
}
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    /// Copy a required field from one selected related subject.
    pub fn initialize_from_related<R, Root, Related, Source, Target>(
        self,
        relation: ApplicationRelationRef<S, R, Root, Related>,
        source: Source,
        target: Target,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Related: OperationReads<O>,
        R: OperationReads<O>,
        Source: RecordSelection<Entity = Related>,
        Target: RecordSelection<Entity = E, Value = Source::Value, Binding = Source::Binding>,
    {
        self.initialize_related(relation, source, target, ApplicationFieldPresence::Required)
    }
    /// Preserve optional absence, present-empty and present values exactly.
    pub fn initialize_optional_from_related<R, Root, Related, Source, Target>(
        self,
        relation: ApplicationRelationRef<S, R, Root, Related>,
        source: Source,
        target: Target,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Related: OperationReads<O>,
        R: OperationReads<O>,
        Source: RecordSelection<Entity = Related>,
        Target: RecordSelection<Entity = E, Value = Source::Value, Binding = Source::Binding>,
    {
        self.initialize_related(relation, source, target, ApplicationFieldPresence::Optional)
    }
    fn initialize_related<R, Root, Related, Source, Target>(
        mut self,
        relation: ApplicationRelationRef<S, R, Root, Related>,
        source: Source,
        target: Target,
        presence: ApplicationFieldPresence,
    ) -> Result<Self, Denial>
    where
        Source: RecordSelection<Entity = Related>,
        Target: RecordSelection<Entity = E, Value = Source::Value, Binding = Source::Binding>,
    {
        let source = super::record_authoring::resolve(source)?;
        let target = super::record_authoring::resolve(target)?;
        if source.presence() != presence || target.presence() != presence {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields.push(Step::RelatedField {
            field: target.locus().clone(),
            binding: target.recipe().binding_identity().clone(),
            scalar_family: target.recipe().scalar_family(),
            source: ApplicationOperationBodyRelatedFieldSource {
                relation: relation.name().into(),
                from: relation.from().into(),
                to: relation.to().into(),
                field: source.locus().clone(),
                presence,
            },
        });
        Ok(self)
    }
}

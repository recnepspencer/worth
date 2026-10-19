//! A typed related preimage is captured by Query, never supplied by the caller.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldPresence, ApplicationFieldRef,
    ApplicationFieldUnit, ApplicationOperationDecisionReadTarget as Read, ApplicationRelationRef,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, OperationReads, OperationWrites,
    OptionalApplicationFieldValue, RequiredApplicationFieldValue,
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
impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Copy a required field of the single selected related subject at this native decision.
    ///
    /// A relation cannot select a different source entity, even with the same value codec:
    /// ```compile_fail
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn wrong_endpoint<S, O, I, Target, Root, Child, Other, R, A, F, TA, TF>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, Target>,
    ///     relation: ApplicationRelationRef<S, R, Root, Other>,
    ///     source: ApplicationFieldRef<S, Child, A, F, String, ReadOnly, EqualityPredicate, NoApplicationUnit>,
    ///     target: ApplicationFieldRef<S, Target, TA, TF, String, ReadOnly, EqualityPredicate, NoApplicationUnit>,
    /// ) where Root: OperationReads<O>, Child: OperationReads<O>, Other: OperationReads<O>, R: OperationReads<O>,
    ///     F: OperationReads<O> + RequiredApplicationFieldValue<Value = String>,
    ///     TF: OperationWrites<O> + RequiredApplicationFieldValue<Value = String>,
    /// {
    ///     let _ = creation.initialize_from_related(relation, source, target);
    /// }
    /// ```
    pub fn initialize_from_related<
        Relation,
        Root,
        Related,
        SourceAspect,
        SourceField,
        SourceWrite,
        SourceEquality,
        TargetAspect,
        TargetField,
        TargetWrite,
        TargetEquality,
        Value,
        Unit,
    >(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Root, Related>,
        source: ApplicationFieldRef<
            Schema,
            Related,
            SourceAspect,
            SourceField,
            Value,
            SourceWrite,
            SourceEquality,
            Unit,
        >,
        target: ApplicationFieldRef<
            Schema,
            Entity,
            TargetAspect,
            TargetField,
            Value,
            TargetWrite,
            TargetEquality,
            Unit,
        >,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<Operation>,
        Related: OperationReads<Operation>,
        Relation: OperationReads<Operation>,
        SourceField: OperationReads<Operation> + RequiredApplicationFieldValue<Value = Value>,
        TargetField: OperationWrites<Operation> + RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.initialize_related(relation, source, target)
    }
    /// Preserve optional absence, present-empty and present values exactly. A missing subject still denies.
    ///
    /// Optional absence cannot initialize a required target:
    /// ```compile_fail
    /// use worth_query_declaration::facade::application_schema::*;
    /// fn wrong_presence<S, O, I, Target, Root, Child, R, A, F, TA, TF>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, Target>,
    ///     relation: ApplicationRelationRef<S, R, Root, Child>,
    ///     source: ApplicationFieldRef<S, Child, A, F, String, ReadOnly, EqualityPredicate, NoApplicationUnit>,
    ///     target: ApplicationFieldRef<S, Target, TA, TF, String, ReadOnly, EqualityPredicate, NoApplicationUnit>,
    /// ) where Root: OperationReads<O>, Child: OperationReads<O>, R: OperationReads<O>,
    ///     F: OperationReads<O> + OptionalApplicationFieldValue<Value = String>,
    ///     TF: OperationWrites<O> + RequiredApplicationFieldValue<Value = String>,
    /// {
    ///     let _ = creation.initialize_optional_from_related(relation, source, target);
    /// }
    /// ```
    pub fn initialize_optional_from_related<
        Relation,
        Root,
        Related,
        SourceAspect,
        SourceField,
        SourceWrite,
        SourceEquality,
        TargetAspect,
        TargetField,
        TargetWrite,
        TargetEquality,
        Value,
        Unit,
    >(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Root, Related>,
        source: ApplicationFieldRef<
            Schema,
            Related,
            SourceAspect,
            SourceField,
            Value,
            SourceWrite,
            SourceEquality,
            Unit,
        >,
        target: ApplicationFieldRef<
            Schema,
            Entity,
            TargetAspect,
            TargetField,
            Value,
            TargetWrite,
            TargetEquality,
            Unit,
        >,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<Operation>,
        Related: OperationReads<Operation>,
        Relation: OperationReads<Operation>,
        SourceField: OperationReads<Operation> + OptionalApplicationFieldValue<Value = Value>,
        TargetField: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.initialize_related(relation, source, target)
    }
    fn initialize_related<
        Relation,
        Root,
        Related,
        SourceAspect,
        SourceField,
        SourceWrite,
        SourceEquality,
        TargetAspect,
        TargetField,
        TargetWrite,
        TargetEquality,
        Value,
        Unit,
    >(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Root, Related>,
        source: ApplicationFieldRef<
            Schema,
            Related,
            SourceAspect,
            SourceField,
            Value,
            SourceWrite,
            SourceEquality,
            Unit,
        >,
        target: ApplicationFieldRef<
            Schema,
            Entity,
            TargetAspect,
            TargetField,
            Value,
            TargetWrite,
            TargetEquality,
            Unit,
        >,
    ) -> Result<Self, Denial>
    where
        SourceField: DeclaredApplicationFieldValue<Value = Value>,
        TargetField: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        if SourceField::Binding::IDENTITY != TargetField::Binding::IDENTITY
            || SourceField::Binding::SCALAR_FAMILY != TargetField::Binding::SCALAR_FAMILY
            || SourceField::PRESENCE != TargetField::PRESENCE
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        self.fields.push(Step::RelatedField {
            field: ApplicationFieldBindingLocus::new(
                target.entity(),
                target.aspect(),
                target.field(),
            ),
            binding: TargetField::Binding::IDENTITY,
            scalar_family: TargetField::Binding::SCALAR_FAMILY,
            source: ApplicationOperationBodyRelatedFieldSource {
                relation: relation.name().into(),
                from: relation.from().into(),
                to: relation.to().into(),
                field: ApplicationFieldBindingLocus::new(
                    source.entity(),
                    source.aspect(),
                    source.field(),
                ),
                presence: SourceField::PRESENCE,
            },
        });
        Ok(self)
    }
}

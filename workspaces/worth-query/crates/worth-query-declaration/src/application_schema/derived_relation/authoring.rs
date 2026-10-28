//! Typed authoring records symbolic dependencies; runtime admission remains separate.
use crate::application_program::{ApplicationDerivedCollection, ApplicationFeature};
use crate::application_schema::{
    ApplicationOperationRef, ApplicationRelationGraphLaws, ApplicationSchema,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember, OperationReads,
};

use super::{
    ApplicationDerivedRelation, ApplicationDerivedRelationRef,
    ApplicationRuleReadableCollectionAssociation, ErasedApplicationDerivedRelationDefinition,
    WorthQueryPortableDerivedRelationParts,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Declares a rebuildable endpoint-set recipe and its graph obligations.
    pub fn derived_relation<Relation, From, To>(
        self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        recipe: ApplicationDerivedRelation<Schema, From, To>,
        laws: ApplicationRelationGraphLaws,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::DerivedRelation {
            definition: ErasedApplicationDerivedRelationDefinition::from_untrusted_parts(
                WorthQueryPortableDerivedRelationParts {
                    relation: relation.name().to_owned(),
                    from: relation.from_member().into_meaning(),
                    to: relation.to_member().into_meaning(),
                    expression: recipe.expression,
                    laws,
                },
            ),
        })
    }

    /// Declares the operation's structural source read; this is not a read permit.
    pub fn operation_read_derived_relation<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationReads<Operation>,
    {
        self.push_member(ApplicationSchemaMember::OperationDerivedRead {
            operation: operation.name().to_owned(),
            relation: relation.name().to_owned(),
        })
    }
}

impl<Schema: ApplicationSchema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Associates the real managed collection identities with one recipe.
    pub fn rule_readable_collection<Feature, Collection, Relation, From, To>(
        self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Feature: ApplicationFeature<Schema>,
        Collection: ApplicationDerivedCollection<Schema, Feature>,
    {
        self.push_member(ApplicationSchemaMember::RuleReadableCollection {
            association: ApplicationRuleReadableCollectionAssociation::of::<
                Schema,
                Feature,
                Collection,
            >(relation.name()),
        })
    }
}

use crate::application_query::ApplicationQueryResultFieldPresence;
use std::marker::PhantomData;

use crate::application_schema::ApplicationScalarValueBinding;
use crate::application_schema::ApplicationStructuredValueBinding;

use super::{
    ApplicationQueryMarkerIdentity, ApplicationQueryOptionalResultFieldRef,
    ApplicationQueryParameterRef, ApplicationQueryPredicate, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationCardinality, ApplicationQueryResultRelationRef,
    ApplicationQueryResultTraversalEndpoints,
};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

mod expression_context;
mod field;
mod optional_aspect;
pub use expression_context::ApplicationQueryExpressionContextDenial;
mod relation;

pub use field::{ApplicationQueryResultField, WorthQueryPortableApplicationQueryResultFieldParts};
pub use relation::{
    ApplicationQueryResultRelation, WorthQueryPortableApplicationQueryResultRelationParts,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryResultShape {
    query_type: WorthQueryPortableTypeIdentity,
    root_entity: String,
    result_type: WorthQueryPortableTypeIdentity,
    fields: Vec<ApplicationQueryResultField>,
    relations: Vec<ApplicationQueryResultRelation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryResultShapeParts {
    pub query_type: WorthQueryPortableTypeIdentity,
    pub root_entity: String,
    pub result_type: WorthQueryPortableTypeIdentity,
    pub fields: Vec<ApplicationQueryResultField>,
    pub relations: Vec<ApplicationQueryResultRelation>,
}

impl ApplicationQueryResultShape {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryResultShapeParts) -> Self {
        Self {
            query_type: parts.query_type,
            root_entity: parts.root_entity,
            result_type: parts.result_type,
            fields: parts.fields,
            relations: parts.relations,
        }
    }

    pub const fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.query_type.clone()
    }

    pub fn root_entity(&self) -> &str {
        &self.root_entity
    }

    pub const fn result_type(&self) -> &str {
        self.result_type.as_str()
    }

    pub fn result_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.result_type.clone()
    }

    pub fn fields(&self) -> &[ApplicationQueryResultField] {
        &self.fields
    }

    pub fn relations(&self) -> &[ApplicationQueryResultRelation] {
        &self.relations
    }
}

pub struct TypedApplicationQueryResultShape<Schema, Query, Entity, Result, ShapeBinding> {
    shape: ApplicationQueryResultShape,
    field_expectations: Vec<crate::application_schema::ApplicationFieldBindingRecipe>,
    _marker: PhantomData<fn() -> (Schema, Query, Entity, Result, ShapeBinding)>,
}

impl<Schema, Query, Entity, Result, ShapeBinding>
    TypedApplicationQueryResultShape<Schema, Query, Entity, Result, ShapeBinding>
{
    pub(crate) fn into_parts(
        self,
    ) -> (
        ApplicationQueryResultShape,
        Vec<crate::application_schema::ApplicationFieldBindingRecipe>,
    ) {
        (self.shape, self.field_expectations)
    }
}

pub struct ApplicationQueryResultShapeBuilder<Schema, Query, Entity, Result, ShapeBinding> {
    root_entity: &'static str,
    field_expectations: Vec<crate::application_schema::ApplicationFieldBindingRecipe>,
    fields: Vec<ApplicationQueryResultField>,
    relations: Vec<ApplicationQueryResultRelation>,
    _marker: PhantomData<fn() -> (Schema, Query, Entity, Result, ShapeBinding)>,
}

impl<Schema, Query, Entity, Result, ShapeBinding>
    ApplicationQueryResultShapeBuilder<Schema, Query, Entity, Result, ShapeBinding>
where
    ShapeBinding: ApplicationStructuredValueBinding<Value = Result>,
{
    pub fn new(root: crate::application_schema::ApplicationEntityRef<Schema, Entity>) -> Self {
        Self {
            root_entity: root.name(),
            field_expectations: Vec::new(),
            fields: Vec::new(),
            relations: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn field<Slot, Aspect, Field, Value>(
        mut self,
        field: ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>,
    ) -> Self
    where
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.field_expectations.push(field.binding_recipe());
        self.fields
            .push(ApplicationQueryResultField::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultFieldParts {
                    query_type: field.slot_key().query_identity(),
                    slot_type: field.slot_key().slot_identity(),
                    entity: field.entity().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                    output_name: field.output_name().to_owned(),
                    scalar_family: Field::Binding::SCALAR_FAMILY,
                    value_type: Field::Binding::IDENTITY,
                    presence: ApplicationQueryResultFieldPresence::Required,
                },
            ));
        self
    }

    pub fn optional_field<Slot, Aspect, Field, Value>(
        mut self,
        field: ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>,
    ) -> Self
    where
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.field_expectations.push(field.binding_recipe());
        self.fields
            .push(ApplicationQueryResultField::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultFieldParts {
                    query_type: field.slot_key().query_identity(),
                    slot_type: field.slot_key().slot_identity(),
                    entity: field.entity().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                    output_name: field.output_name().to_owned(),
                    scalar_family: Field::Binding::SCALAR_FAMILY,
                    value_type: Field::Binding::IDENTITY,
                    presence: ApplicationQueryResultFieldPresence::Optional,
                },
            ));
        self
    }

    pub fn relation<
        Slot,
        Relation,
        DeclaredFrom,
        DeclaredTo,
        Direction,
        Child,
        Cardinality,
        NestedResult,
        NestedBinding,
    >(
        mut self,
        relation: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            DeclaredFrom,
            DeclaredTo,
            Direction,
            Cardinality,
        >,
        nested: ApplicationQueryResultShapeBuilder<
            Schema,
            Query,
            Child,
            NestedResult,
            NestedBinding,
        >,
    ) -> Self
    where
        Direction:
            ApplicationQueryResultTraversalEndpoints<Entity, Child, DeclaredFrom, DeclaredTo>,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        NestedBinding: ApplicationStructuredValueBinding<Value = NestedResult>,
        Slot: WorthQueryPortableType,
    {
        let (nested_shape, nested_fields) = nested.build().into_parts();
        self.field_expectations.extend(nested_fields);
        self.relations
            .push(ApplicationQueryResultRelation::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultRelationParts {
                    query_type: relation.slot_key().query_identity(),
                    slot_type: relation.slot_key().slot_identity(),
                    relation: relation.relation().to_owned(),
                    from: relation.from().to_owned(),
                    to: relation.to().to_owned(),
                    direction: relation.direction(),
                    output_name: relation.output_name().to_owned(),
                    cardinality: relation.cardinality(),
                    predicate: None,
                    nested_shape,
                },
            ));
        self
    }

    pub fn relation_where_equal<
        Slot,
        Relation,
        DeclaredFrom,
        DeclaredTo,
        Direction,
        Child,
        Cardinality,
        NestedResult,
        NestedBinding,
        Field,
        Parameter,
        Binding,
    >(
        mut self,
        relation: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            DeclaredFrom,
            DeclaredTo,
            Direction,
            Cardinality,
        >,
        nested: ApplicationQueryResultShapeBuilder<
            Schema,
            Query,
            Child,
            NestedResult,
            NestedBinding,
        >,
        field: Field,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
    ) -> Self
    where
        Direction:
            ApplicationQueryResultTraversalEndpoints<Entity, Child, DeclaredFrom, DeclaredTo>,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        NestedBinding: ApplicationStructuredValueBinding<Value = NestedResult>,
        Slot: WorthQueryPortableType,
        Binding: crate::application_schema::ApplicationReadableScalarValueBinding,
        Field:
            crate::application_schema::fragment::RecordSelection<Entity = Child, Binding = Binding>,
    {
        self.field_expectations.push(field.expected_recipe());
        let (nested_shape, nested_fields) = nested.build().into_parts();
        self.field_expectations.extend(nested_fields);
        self.relations
            .push(ApplicationQueryResultRelation::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultRelationParts {
                    query_type: relation.slot_key().query_identity(),
                    slot_type: relation.slot_key().slot_identity(),
                    relation: relation.relation().to_owned(),
                    from: relation.from().to_owned(),
                    to: relation.to().to_owned(),
                    direction: relation.direction(),
                    output_name: relation.output_name().to_owned(),
                    cardinality: relation.cardinality(),
                    predicate: Some(ApplicationQueryPredicate::from_untrusted_fields(
                        field.entity().to_owned(),
                        field.aspect().to_owned(),
                        field.field().to_owned(),
                        parameter.name().to_owned(),
                        Binding::SCALAR_FAMILY,
                    )),
                    nested_shape,
                },
            ));
        self
    }

    pub fn build(
        mut self,
    ) -> TypedApplicationQueryResultShape<Schema, Query, Entity, Result, ShapeBinding>
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
    {
        self.fields.sort();
        self.relations.sort();
        TypedApplicationQueryResultShape {
            field_expectations: self.field_expectations,
            shape: ApplicationQueryResultShape {
                query_type: Query::QUERY_TYPE_IDENTITY,
                root_entity: self.root_entity.to_owned(),
                result_type: ShapeBinding::IDENTITY,
                fields: self.fields,
                relations: self.relations,
            },
            _marker: PhantomData,
        }
    }
}

use std::marker::PhantomData;

use crate::application_schema::ApplicationStructuredValueBinding;
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationFieldUnit, ApplicationScalarValueBinding,
    OptionalApplicationFieldValue, RequiredApplicationFieldValue,
};

use super::{
    ApplicationQueryMarkerIdentity, ApplicationQueryOptionalResultFieldRef,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationCardinality,
    ApplicationQueryResultRelationRef, ApplicationQueryResultTraversalEndpoints,
};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

mod field;
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
    _marker: PhantomData<fn() -> (Schema, Query, Entity, Result, ShapeBinding)>,
}

impl<Schema, Query, Entity, Result, ShapeBinding>
    TypedApplicationQueryResultShape<Schema, Query, Entity, Result, ShapeBinding>
{
    pub(crate) fn into_erased(self) -> ApplicationQueryResultShape {
        self.shape
    }
}

pub struct ApplicationQueryResultShapeBuilder<Schema, Query, Entity, Result, ShapeBinding> {
    root_entity: &'static str,
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
            fields: Vec::new(),
            relations: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn field<Slot, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Self
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
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
                    presence: ApplicationFieldPresence::Required,
                },
            ));
        self
    }

    pub fn optional_field<Slot, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationQueryOptionalResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Self
    where
        Field: OptionalApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
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
                    presence: ApplicationFieldPresence::Optional,
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
                    nested_shape: nested.build().into_erased(),
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

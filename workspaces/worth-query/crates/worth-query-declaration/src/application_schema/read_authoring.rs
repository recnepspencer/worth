use std::marker::PhantomData;

use worth_foundational::facade::AspectValue;

use super::authoring_context::ApplicationFieldAdmission;
use super::capabilities::{ApplicationFieldUnit, EqualityCapable, EqualityPredicate};
use super::field_reference::ApplicationFieldRef;
use super::references::{ApplicationEntityRef, ApplicationRelationRef};
use super::values::DeclaredApplicationFieldValue;
use super::ApplicationEncodedScalarValue;
use super::{
    ApplicationSchemaAuthoringContext, ApplicationSchemaAuthoringDenial,
    ApplicationSchemaBindingIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedProjection {
    aspect: &'static str,
    field: &'static str,
}

impl TypedProjection {
    pub const fn aspect(&self) -> &'static str {
        self.aspect
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedEqualityPredicate {
    aspect: &'static str,
    field: &'static str,
    value: AspectValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedTraversal {
    relation: &'static str,
    from: &'static str,
    to: &'static str,
}

impl TypedTraversal {
    pub const fn relation(&self) -> &'static str {
        self.relation
    }

    pub const fn from(&self) -> &'static str {
        self.from
    }

    pub const fn to(&self) -> &'static str {
        self.to
    }
}

impl TypedEqualityPredicate {
    pub const fn aspect(&self) -> &'static str {
        self.aspect
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }
}

pub struct TypedReadDeclaration<Schema, Entity> {
    binding: Option<ApplicationSchemaBindingIdentity>,
    entity: &'static str,
    current_entity: &'static str,
    traversals: Vec<TypedTraversal>,
    projections: Vec<TypedProjection>,
    predicates: Vec<TypedEqualityPredicate>,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> Clone for TypedReadDeclaration<Schema, Entity> {
    fn clone(&self) -> Self {
        Self {
            binding: self.binding.clone(),
            entity: self.entity,
            current_entity: self.current_entity,
            traversals: self.traversals.clone(),
            projections: self.projections.clone(),
            predicates: self.predicates.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Entity> std::fmt::Debug for TypedReadDeclaration<Schema, Entity> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TypedReadDeclaration")
            .field("binding", &self.binding)
            .field("entity", &self.entity)
            .field("current_entity", &self.current_entity)
            .field("traversals", &self.traversals)
            .field("projections", &self.projections)
            .field("predicates", &self.predicates)
            .finish_non_exhaustive()
    }
}

impl<Schema, Entity> PartialEq for TypedReadDeclaration<Schema, Entity> {
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding
            && self.entity == other.entity
            && self.current_entity == other.current_entity
            && self.traversals == other.traversals
            && self.projections == other.projections
            && self.predicates == other.predicates
    }
}

impl<Schema, Entity> TypedReadDeclaration<Schema, Entity> {
    pub fn binding(&self) -> Option<&ApplicationSchemaBindingIdentity> {
        self.binding.as_ref()
    }

    pub const fn entity(&self) -> &'static str {
        self.entity
    }

    pub const fn current_entity(&self) -> &'static str {
        self.current_entity
    }

    pub fn traversals(&self) -> &[TypedTraversal] {
        &self.traversals
    }

    pub fn projections(&self) -> &[TypedProjection] {
        &self.projections
    }

    pub fn predicates(&self) -> &[TypedEqualityPredicate] {
        &self.predicates
    }
}

pub struct TypedReadDeclarationBuilder<Schema, Entity> {
    context: Option<ApplicationSchemaAuthoringContext>,
    denial: Option<ApplicationSchemaAuthoringDenial>,
    entity: &'static str,
    current_entity: &'static str,
    traversals: Vec<TypedTraversal>,
    projections: Vec<TypedProjection>,
    predicates: Vec<TypedEqualityPredicate>,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> TypedReadDeclarationBuilder<Schema, Entity> {
    pub fn new(entity: ApplicationEntityRef<Schema, Entity>) -> Self {
        Self {
            context: None,
            denial: None,
            entity: entity.name(),
            current_entity: entity.name(),
            traversals: Vec::new(),
            projections: Vec::new(),
            predicates: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn traverse<Relation, To>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, To>,
    ) -> TypedReadDeclarationBuilder<Schema, To> {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_relation(relation.name(), relation.from(), relation.to())
                    .err()
            });
        }
        self.traversals.push(TypedTraversal {
            relation: relation.name(),
            from: relation.from(),
            to: relation.to(),
        });
        TypedReadDeclarationBuilder {
            context: self.context,
            denial: self.denial,
            entity: self.entity,
            current_entity: relation.to(),
            traversals: self.traversals,
            projections: self.projections,
            predicates: self.predicates,
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn with_installed_context(mut self, context: ApplicationSchemaAuthoringContext) -> Self {
        self.denial = context.admit_entity(self.entity).err();
        self.context = Some(context);
        self
    }

    pub fn project<Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_field(ApplicationFieldAdmission {
                        entity: field.entity(),
                        aspect: field.aspect(),
                        field: field.field(),
                        scalar_family: field.scalar_family(),
                        value_type: field.value_type_name(),
                        unit: field.unit(),
                        requires_write: false,
                        requires_equality: false,
                    })
                    .err()
            });
        }
        self.projections.push(TypedProjection {
            aspect: field.aspect(),
            field: field.field(),
        });
        self
    }

    pub fn where_equal<Aspect, Field, Value, Write, Unit>(
        mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        EqualityPredicate: EqualityCapable,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_field(ApplicationFieldAdmission {
                        entity: field.entity(),
                        aspect: field.aspect(),
                        field: field.field(),
                        scalar_family: field.scalar_family(),
                        value_type: field.value_type_name(),
                        unit: field.unit(),
                        requires_write: false,
                        requires_equality: true,
                    })
                    .err()
            });
        }
        self.predicates.push(TypedEqualityPredicate {
            aspect: field.aspect(),
            field: field.field(),
            value: value.into_foundational_value(),
        });
        self
    }

    pub fn build(
        self,
    ) -> Result<TypedReadDeclaration<Schema, Entity>, ApplicationSchemaAuthoringDenial> {
        if let Some(denial) = self.denial {
            return Err(denial);
        }
        Ok(TypedReadDeclaration {
            binding: self.context.map(|context| context.binding().clone()),
            entity: self.entity,
            current_entity: self.current_entity,
            traversals: self.traversals,
            projections: self.projections,
            predicates: self.predicates,
            _marker: PhantomData,
        })
    }
}

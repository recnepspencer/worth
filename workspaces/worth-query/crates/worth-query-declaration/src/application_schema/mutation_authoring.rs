use std::marker::PhantomData;

use worth_foundational::facade::AspectValue;

use super::authoring_context::{ApplicationFieldAdmission, ApplicationOperationProgramAdmission};
use super::capabilities::{
    ApplicationFieldUnit, CreatableBy, OperationCreates, OperationDeletes, OperationLinks,
    OperationUnlinks, OperationWrites, WritableCapability,
};
use super::field_reference::ApplicationFieldRef;
use super::references::{ApplicationEntityRef, ApplicationOperationRef, ApplicationRelationRef};
use super::values::DeclaredApplicationFieldValue;
use super::{
    ApplicationEncodedScalarValue, ApplicationOperationMarkerIdentity,
    ApplicationSchemaAuthoringContext, ApplicationSchemaAuthoringDenial,
    ApplicationSchemaBindingIdentity, ApplicationStructuredValueBinding,
};

pub struct TypedOperationBuilder<Schema, Operation, Input> {
    operation: &'static str,
    context: Option<ApplicationSchemaAuthoringContext>,
    denial: Option<ApplicationSchemaAuthoringDenial>,
    _marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

impl<Schema, Operation: 'static, Input> TypedOperationBuilder<Schema, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: 'static,
{
    pub fn new(operation: ApplicationOperationRef<Schema, Operation, Input>) -> Self {
        Self {
            operation: operation.name(),
            context: None,
            denial: None,
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn with_installed_context(mut self, context: ApplicationSchemaAuthoringContext) -> Self {
        self.denial = context
            .admit_operation::<Operation, Input>(self.operation, Operation::InputBinding::IDENTITY)
            .err();
        self.context = Some(context);
        self
    }

    pub fn input(self, input: Input) -> TypedMutationIntentBuilder<Schema, Operation, Input> {
        TypedMutationIntentBuilder {
            operation: self.operation,
            context: self.context,
            denial: self.denial,
            input,
            creates: Vec::new(),
            deletes: Vec::new(),
            links: Vec::new(),
            unlinks: Vec::new(),
            writes: Vec::new(),
            _marker: PhantomData,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedMutationWrite {
    aspect: &'static str,
    field: &'static str,
    value: AspectValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedRelationMutation {
    relation: &'static str,
    from: &'static str,
    to: &'static str,
}

impl TypedRelationMutation {
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

impl TypedMutationWrite {
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

pub struct TypedMutationIntent<Schema, Operation, Input> {
    pub(super) operation: &'static str,
    pub(super) binding: Option<ApplicationSchemaBindingIdentity>,
    pub(super) input: Input,
    pub(super) creates: Vec<&'static str>,
    pub(super) deletes: Vec<&'static str>,
    pub(super) links: Vec<TypedRelationMutation>,
    pub(super) unlinks: Vec<TypedRelationMutation>,
    pub(super) writes: Vec<TypedMutationWrite>,
    pub(super) _marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema, Operation, Input> TypedMutationIntent<Schema, Operation, Input> {
    pub fn binding(&self) -> Option<&ApplicationSchemaBindingIdentity> {
        self.binding.as_ref()
    }

    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn creates(&self) -> &[&'static str] {
        &self.creates
    }

    pub fn deletes(&self) -> &[&'static str] {
        &self.deletes
    }

    pub fn links(&self) -> &[TypedRelationMutation] {
        &self.links
    }

    pub fn unlinks(&self) -> &[TypedRelationMutation] {
        &self.unlinks
    }

    pub fn writes(&self) -> &[TypedMutationWrite] {
        &self.writes
    }
}

pub struct TypedMutationIntentBuilder<Schema, Operation, Input> {
    operation: &'static str,
    context: Option<ApplicationSchemaAuthoringContext>,
    denial: Option<ApplicationSchemaAuthoringDenial>,
    input: Input,
    creates: Vec<&'static str>,
    deletes: Vec<&'static str>,
    links: Vec<TypedRelationMutation>,
    unlinks: Vec<TypedRelationMutation>,
    writes: Vec<TypedMutationWrite>,
    _marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema, Operation, Input> TypedMutationIntentBuilder<Schema, Operation, Input> {
    pub fn create<Entity>(mut self, entity: ApplicationEntityRef<Schema, Entity>) -> Self
    where
        Entity: OperationCreates<Operation> + CreatableBy<Operation>,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_entity(entity.name())
                    .and_then(|()| {
                        context.admit_operation_program(
                            self.operation,
                            ApplicationOperationProgramAdmission::Create(entity.name()),
                        )
                    })
                    .err()
            });
        }
        self.creates.push(entity.name());
        self
    }

    pub fn delete<Entity>(mut self, entity: ApplicationEntityRef<Schema, Entity>) -> Self
    where
        Entity: OperationDeletes<Operation>,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_entity(entity.name())
                    .and_then(|()| {
                        context.admit_operation_program(
                            self.operation,
                            ApplicationOperationProgramAdmission::Delete(entity.name()),
                        )
                    })
                    .err()
            });
        }
        self.deletes.push(entity.name());
        self
    }

    pub fn link<Relation, From, To>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationLinks<Operation>,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_relation(relation.name(), relation.from(), relation.to())
                    .and_then(|()| {
                        context.admit_operation_program(
                            self.operation,
                            ApplicationOperationProgramAdmission::Link {
                                relation: relation.name(),
                                from: relation.from(),
                                to: relation.to(),
                            },
                        )
                    })
                    .err()
            });
        }
        self.links.push(relation_mutation(relation));
        self
    }

    pub fn unlink<Relation, From, To>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationUnlinks<Operation>,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_relation(relation.name(), relation.from(), relation.to())
                    .and_then(|()| {
                        context.admit_operation_program(
                            self.operation,
                            ApplicationOperationProgramAdmission::Unlink {
                                relation: relation.name(),
                                from: relation.from(),
                                to: relation.to(),
                            },
                        )
                    })
                    .err()
            });
        }
        self.unlinks.push(relation_mutation(relation));
        self
    }

    pub fn set<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
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
                        requires_write: true,
                        requires_equality: false,
                    })
                    .and_then(|()| {
                        context.admit_operation_program(
                            self.operation,
                            ApplicationOperationProgramAdmission::Write {
                                entity: field.entity(),
                                aspect: field.aspect(),
                                field: field.field(),
                            },
                        )
                    })
                    .err()
            });
        }
        self.writes.push(TypedMutationWrite {
            aspect: field.aspect(),
            field: field.field(),
            value: value.into_foundational_value(),
        });
        self
    }

    pub fn build(
        self,
    ) -> Result<TypedMutationIntent<Schema, Operation, Input>, ApplicationSchemaAuthoringDenial>
    {
        if let Some(denial) = self.denial {
            return Err(denial);
        }
        Ok(TypedMutationIntent {
            operation: self.operation,
            binding: self.context.map(|context| context.binding().clone()),
            input: self.input,
            creates: self.creates,
            deletes: self.deletes,
            links: self.links,
            unlinks: self.unlinks,
            writes: self.writes,
            _marker: PhantomData,
        })
    }
}

fn relation_mutation<Schema, Relation, From, To>(
    relation: ApplicationRelationRef<Schema, Relation, From, To>,
) -> TypedRelationMutation {
    TypedRelationMutation {
        relation: relation.name(),
        from: relation.from(),
        to: relation.to(),
    }
}

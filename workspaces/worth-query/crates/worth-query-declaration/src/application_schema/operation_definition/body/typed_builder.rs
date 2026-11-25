//! Chained authored-body builders that accept compile-time-typed input fields
//! and propagate the first [`Denial`] fail-closed through `.finish()`.

use crate::application_query::ApplicationQueryRootPath;
use crate::application_schema::{
    fragment::RecordSelection, ApplicationEntityRef, ApplicationIdentityScalarValueBinding,
    ApplicationOperationDefinition, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationRelationRef, ApplicationStructuredValueBinding, OperationCreates, OperationLinks,
    OperationReads, OperationUnlinks,
};

use super::typed_input::{ApplicationInputField, ApplicationOptionalInputField};
use super::{
    ApplicationOperationBody, ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyBuilder, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyRequirement,
};

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    /// Begins a chained, compile-time-typed authored body builder that carries
    /// the first authoring denial through to [`TypedApplicationOperationBodyBuilder::finish`].
    pub fn typed_body(self) -> TypedApplicationOperationBodyBuilder<Schema, Operation, Input> {
        TypedApplicationOperationBodyBuilder {
            operation: self,
            state: Ok(self.body()),
        }
    }

    /// Wraps a finished authored body as a pure (no external effect, no aftermath)
    /// operation definition.
    pub fn pure_body_definition(
        self,
        body: ApplicationOperationBody<Schema, Operation, Input>,
    ) -> Result<ApplicationOperationDefinition<Schema, Operation, Input>, Denial> {
        self.definition()
            .no_external_effect()
            .no_aftermath()
            .finish()
            .with_body(body)
    }
}

/// Chained authored-body builder accepting compile-time-typed input fields and
/// propagating the first [`Denial`] fail-closed to `.finish()`.
pub struct TypedApplicationOperationBodyBuilder<Schema, Operation, Input> {
    operation: ApplicationOperationRef<Schema, Operation, Input>,
    state: Result<ApplicationOperationBodyBuilder<Schema, Operation, Input>, Denial>,
}

impl<Schema, Operation, Input> TypedApplicationOperationBodyBuilder<Schema, Operation, Input> {
    fn step(
        self,
        apply: impl FnOnce(
            ApplicationOperationBodyBuilder<Schema, Operation, Input>,
        )
            -> Result<ApplicationOperationBodyBuilder<Schema, Operation, Input>, Denial>,
    ) -> Self {
        Self {
            operation: self.operation,
            state: self.state.and_then(apply),
        }
    }

    pub fn in_scope<Entity: OperationReads<Operation>>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self {
        self.step(|builder| builder.in_scope(entity))
    }

    pub fn expect<Field: RecordSelection>(self, field: Field, value: Field::Value) -> Self {
        self.step(|builder| builder.expect(field, value))
    }

    pub fn expect_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        self.step(|builder| builder.expect_typed_input(field, input))
    }

    pub fn assign<Field: RecordSelection>(self, field: Field, value: Field::Value) -> Self {
        self.step(|builder| builder.assign(field, value))
    }

    pub fn assign_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        self.step(|builder| builder.assign_typed_input(field, input))
    }

    pub fn set_optional_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationInputField<Input, Field::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        self.step(|builder| builder.set_typed_optional_input(field, input))
    }

    pub fn assign_optional_input<Field: RecordSelection>(
        self,
        field: Field,
        input: ApplicationOptionalInputField<Input, Field::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        self.step(|builder| builder.assign_typed_optional_input(field, input))
    }

    pub fn clear_optional<Field: RecordSelection>(self, field: Field) -> Self {
        self.step(|builder| builder.clear_optional(field))
    }

    pub fn require_present<Field: RecordSelection>(self, field: Field) -> Self {
        self.step(|builder| builder.require_present(field))
    }

    pub fn requirement(
        self,
        requirement: ApplicationOperationBodyRequirement<Schema, Operation>,
    ) -> Self {
        self.step(|builder| builder.requirement(requirement))
    }

    pub fn expect_input_from_path<Root, Selected, F>(
        self,
        path: ApplicationQueryRootPath<Schema, Root, Selected>,
        field: F,
        input: ApplicationInputField<Input, F::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Root: OperationReads<Operation>,
        Selected: OperationReads<Operation>,
        F: RecordSelection<Entity = Selected>,
    {
        self.step(|builder| builder.expect_typed_input_from_path(path, field, input))
    }

    pub fn replace_relation_from_input<R, Root, Selected, F>(
        self,
        relation: ApplicationRelationRef<Schema, R, Root, Selected>,
        identity: F,
        input: ApplicationOptionalInputField<Input, F::Value>,
        key: &str,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Root: OperationReads<Operation>,
        Selected: OperationReads<Operation>,
        R: OperationReads<Operation> + OperationLinks<Operation> + OperationUnlinks<Operation>,
        F: RecordSelection<Entity = Selected>,
        F::Binding: ApplicationIdentityScalarValueBinding,
    {
        self.step(|builder| {
            builder.replace_relation_from_typed_input(relation, identity, input, key)
        })
    }

    pub fn create_in_scope<Entity: OperationCreates<Operation>>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: &str,
    ) -> TypedApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity> {
        TypedApplicationOperationBodyCreationBuilder {
            operation: self.operation,
            state: self
                .state
                .and_then(|builder| builder.create_in_scope(entity, key)),
        }
    }

    /// Returns the underlying low-level builder when specialized steps are needed.
    pub fn into_inner(
        self,
    ) -> Result<ApplicationOperationBodyBuilder<Schema, Operation, Input>, Denial> {
        self.state
    }

    pub fn finish(self) -> Result<ApplicationOperationBody<Schema, Operation, Input>, Denial> {
        self.state?.finish()
    }

    /// Finishes the body and attaches it to a pure (no external effect, no aftermath)
    /// operation definition in one step.
    pub fn finish_pure_definition(
        self,
    ) -> Result<ApplicationOperationDefinition<Schema, Operation, Input>, Denial> {
        let operation = self.operation;
        let body = self.state?.finish()?;
        operation.pure_body_definition(body)
    }
}

/// Chained creation builder entered from [`TypedApplicationOperationBodyBuilder::create_in_scope`].
pub struct TypedApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity> {
    operation: ApplicationOperationRef<Schema, Operation, Input>,
    state:
        Result<ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>, Denial>,
}

impl<Schema, Operation, Input, Entity>
    TypedApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    fn step(
        self,
        apply: impl FnOnce(
            ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>,
        ) -> Result<
            ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>,
            Denial,
        >,
    ) -> Self {
        Self {
            operation: self.operation,
            state: self.state.and_then(apply),
        }
    }

    pub fn initialize<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        value: Selection::Value,
    ) -> Self {
        self.step(|builder| builder.initialize(field, value))
    }

    pub fn initialize_input<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        input: ApplicationInputField<Input, Selection::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        self.step(|builder| builder.initialize_typed_input(field, input))
    }

    pub fn initialize_optional_input<Selection: RecordSelection<Entity = Entity>>(
        self,
        field: Selection,
        input: ApplicationOptionalInputField<Input, Selection::Value>,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        self.step(|builder| builder.initialize_typed_optional_input(field, input))
    }

    pub fn initialize_from_scope<Source, Field>(self, source: Source, field: Field) -> Self
    where
        Source: RecordSelection,
        Field: RecordSelection<Entity = Entity, Value = Source::Value, Binding = Source::Binding>,
    {
        self.step(|builder| builder.initialize_from_scope(source, field))
    }

    pub fn link_from_scope<Relation: OperationLinks<Operation>, Scope>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Scope, Entity>,
        key: &str,
    ) -> Self {
        self.step(|builder| builder.link_from_scope(relation, key))
    }

    pub fn link_to_scope<Relation: OperationLinks<Operation>, Scope>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Scope>,
        key: &str,
    ) -> Self {
        self.step(|builder| builder.link_to_scope(relation, key))
    }

    pub fn replace_from_scope<Relation, Scope>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Scope, Entity>,
        key: &str,
    ) -> Self
    where
        Relation:
            OperationReads<Operation> + OperationLinks<Operation> + OperationUnlinks<Operation>,
    {
        self.step(|builder| builder.replace_from_scope(relation, key))
    }

    /// Returns the underlying low-level creation builder when specialized steps are needed.
    pub fn into_inner(
        self,
    ) -> Result<ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>, Denial>
    {
        self.state
    }

    pub fn finish(self) -> Result<ApplicationOperationBody<Schema, Operation, Input>, Denial> {
        self.state?.finish()
    }

    /// Finishes the creation body and attaches it to a pure operation definition.
    pub fn finish_pure_definition(
        self,
    ) -> Result<ApplicationOperationDefinition<Schema, Operation, Input>, Denial> {
        let operation = self.operation;
        let body = self.state?.finish()?;
        operation.pure_body_definition(body)
    }
}

//! Crate-boundary internal port implementation for `EffectProgramBuilderAuthoringInternalPort`.

use super::EffectProgramBuilderAuthoringInternalPort;
use std::borrow::Cow;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    fragment::{RecordSelection, RelationSelection},
    ApplicationContentBinding, ApplicationContentFieldRef, ApplicationContentReferenceBinding,
    ApplicationEncodedScalarValue, ApplicationFieldBindingLocus,
    ApplicationOperationBodySigningSpec, ApplicationSchema, OperationCreates, OperationLinks,
    OperationReads, OperationUnlinks,
};
use worth_query_installation::facade::{
    ApplicationOperationProgramTarget, ApplicationRelationRef,
    WorthQueryInstalledApplicationValueBindingCatalog,
};

use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    core::selection::EffectSelection, output_correspondence::OutputRoleUse,
    WorthQueryApplicationCreatedRelation, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::operation_body::emission::BodyExternalEffectSelection;
use crate::domain_computation::primary_graph::ContentWriteDenial;

impl<Schema, Operation, Input, Scope>
    EffectProgramBuilderAuthoringInternalPort<Schema, Operation, Input, Scope>
    for WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    fn handler_checkpoint(
        &self,
    ) -> Result<
        (),
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    > {
        Self::handler_checkpoint(self)
    }

    fn initialize_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: Cow<'_, AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::initialize_body_field(self, target, field, value)
    }

    fn observe_unique_write_dependencies(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::observe_unique_write_dependencies(self, execution, allocation)
    }

    fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial> {
        Self::field_locator(self, entity, aspect, field)
    }

    fn prepare_output_contract<Binding>(&mut self) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        Self::prepare_output_contract::<Binding>(self)
    }

    fn bind_output<Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::bind_output(self, role, target)
    }

    fn body_external_emission(
        &mut self,
        selection: BodyExternalEffectSelection,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::body_external_emission(self, selection)
    }

    fn body_effect_emission(
        &mut self,
        recipe: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyEffectRecipe,
        input: &dyn std::any::Any,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::body_effect_emission(self, recipe, input)
    }

    fn create_body_entity_in_context<ContextEntity>(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: &str,
        key: &str,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, ()>, WorthQueryApplicationAttemptDenial>
    {
        Self::create_body_entity_in_context(self, context, entity, key)
    }

    fn delete_at_target<E>(
        &mut self,
        program_target: &ApplicationOperationProgramTarget,
        target: &WorthQueryApplicationEffectEntity<Schema, E>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::delete_at_target(self, program_target, target)
    }

    fn initialize_content<E, F, C>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
        target: &WorthQueryApplicationEffectEntity<Schema, E>,
        content: ApplicationContentFieldRef<F, C>,
        value: &C::Value,
    ) -> Result<(), ContentWriteDenial>
    where
        E: OperationCreates<Operation>,
        F: RecordSelection<Entity = E>,
        F::Binding: ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        Self::initialize_content(self, execution, bindings, target, content, value)
    }

    fn write_content<E, F, C>(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
        target: &WorthQueryApplicationEffectEntity<Schema, E>,
        content: ApplicationContentFieldRef<F, C>,
        value: &C::Value,
    ) -> Result<(), ContentWriteDenial>
    where
        F: RecordSelection<Entity = E>,
        F::Binding: ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        Self::write_content(self, execution, bindings, target, content, value)
    }

    fn link_for_initialization<Relation, From, To>(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<
        WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Relation: OperationLinks<Operation>,
    {
        Self::link_for_initialization(self, original, relation, key, from, to)
    }

    fn initialize_relation_field<Relation, From, To, Field>(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
        target: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        field: Field,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Operation>,
        Field: RelationSelection<Entity = Relation, From = From, To = To>,
    {
        Self::initialize_relation_field(self, original, bindings, target, field, value)
    }

    fn unlink_observed<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationReads<Operation> + OperationUnlinks<Operation>,
    {
        Self::unlink_observed(self, relation, from, to)
    }

    fn unlink_at_target<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::unlink_at_target(self, target, from, to)
    }

    fn link_at_target<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::link_at_target(self, target, key, from, to)
    }

    fn native_selection(&self) -> EffectSelection<'_> {
        Self::native_selection(self)
    }

    fn resolve_observed_entity<Entity, Aspect, Field, Value>(
        &self,
        field: Field,
        value: Value,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
    {
        Self::resolve_observed_entity(self, field, value)
    }

    fn reserve_body_signing_slot(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, ()>,
        spec: &ApplicationOperationBodySigningSpec,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::reserve_body_signing_slot(self, target, spec)
    }

    fn write_body_input_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::write_body_input_field(self, target, field, value)
    }

    fn write_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: &AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::write_body_field(self, target, field, value)
    }

    fn write_body_optional_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: Option<AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::write_body_optional_field(self, target, field, value)
    }
}

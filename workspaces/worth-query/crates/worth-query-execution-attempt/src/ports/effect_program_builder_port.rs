//! Crate-boundary internal port for `WorthQueryApplicationEffectProgramBuilder`
//! authoring operations consumed by `worth-query-execution-query` and
//! `worth-query-execution`. Never exported on `worth-query-execution::facade` or Host.

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
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::operation_body::emission::BodyExternalEffectSelection;
use crate::domain_computation::primary_graph::ContentWriteDenial;

pub trait EffectProgramBuilderAuthoringInternalPort<Schema, Operation, Input, Scope> {
    fn handler_checkpoint(
        &self,
    ) -> Result<
        (),
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    >;

    fn initialize_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: Cow<'_, AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;


    fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial>;

    fn prepare_output_contract<Binding>(
        &mut self,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>;

    fn bind_output<Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn body_external_emission(
        &mut self,
        selection: BodyExternalEffectSelection,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn body_effect_emission(
        &mut self,
        recipe: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyEffectRecipe,
        input: &dyn std::any::Any,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn create_body_entity_in_context<ContextEntity>(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: &str,
        key: &str,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, ()>, WorthQueryApplicationAttemptDenial>;

    fn delete_at_target<E>(
        &mut self,
        program_target: &ApplicationOperationProgramTarget,
        target: &WorthQueryApplicationEffectEntity<Schema, E>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

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
        C: ApplicationContentBinding;

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
        C: ApplicationContentBinding;

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
        Relation: OperationLinks<Operation>;

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
        Field: RelationSelection<Entity = Relation, From = From, To = To>;

    fn unlink_observed<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationReads<Operation> + OperationUnlinks<Operation>;

    fn unlink_at_target<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn link_at_target<From, To>(
        &mut self,
        target: &ApplicationOperationProgramTarget,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn native_selection(&self) -> EffectSelection<'_>;

    fn resolve_observed_entity<Entity, Aspect, Field, Value>(
        &self,
        field: Field,
        value: Value,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>;

    fn reserve_body_signing_slot(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, ()>,
        spec: &ApplicationOperationBodySigningSpec,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn write_body_input_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn write_body_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: &AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;

    fn write_body_optional_field<Entity>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        value: Option<AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;
}

#[path = "effect_program_builder_impl.rs"]
mod impls;

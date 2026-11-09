use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationExternalEffectBinding,
    ApplicationStructuredValueBinding, OperationEmits,
};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationRelationRef, OperationCreates, OperationDeletes,
    OperationLinks,
};

use super::CandidateWriter;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEntityKey,
};

impl<Schema, Binding> CandidateWriter<'_, Schema, Binding>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    #[inline(always)]
    pub fn create_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Binding::Operation>,
    {
        self.candidate.create_entity(entity, key)
    }

    #[inline(always)]
    pub fn create_entity_in_context<Entity, ContextEntity>(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Binding::Operation>,
    {
        self.candidate
            .create_entity_in_context(context, entity, key)
    }

    #[inline(always)]
    pub fn initialize_field<Entity, Aspect, Field, Value>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: Field,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Binding::Operation>,
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        self.candidate.initialize_field(target, field, value)
    }

    #[inline(always)]
    pub fn write_field<Entity, Aspect, Field, Value>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: Field,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        self.candidate.write_field(target, field, value)
    }

    /// Writes or clears one optional field using the same observed target and
    /// effect authority as every other handler-authored effect.
    #[inline(always)]
    pub fn write_optional_field<Entity, Aspect, Field, Value>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: Field,
        value: Option<Value>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        self.candidate.write_optional_field(target, field, value)
    }

    #[inline(always)]
    pub fn link<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Binding::Operation>,
    {
        self.candidate.link(relation, key, from, to)
    }

    pub fn delete_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationDeletes<Binding::Operation>,
    {
        self.candidate.delete_entity(entity, target)
    }

    pub fn emit<Effect, Payload>(
        &mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        payload: Payload,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Binding::Operation>,
        Effect::PayloadBinding: ApplicationStructuredValueBinding<Value = Payload>,
        Payload: Send + Sync + 'static,
    {
        self.candidate.emit(effect, payload)
    }

    pub fn emit_external<Effect, Payload>(
        &mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        payload: Payload,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Binding::Operation>,
        Effect::PayloadBinding: ApplicationExternalEffectBinding<Value = Payload>,
        Payload: Send + Sync + 'static,
    {
        self.candidate.emit_external(effect, payload)
    }
}

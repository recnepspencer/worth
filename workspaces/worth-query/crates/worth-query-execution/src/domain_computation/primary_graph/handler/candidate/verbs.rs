use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationExternalEffectBinding,
    ApplicationRetainedEffectBinding, OperationEmits,
};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef,
    DeclaredApplicationFieldValue, OperationCreates, OperationDeletes, OperationLinks,
    OperationWrites, WritableCapability,
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

    pub fn initialize_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Binding::Operation>,
        Field: OperationWrites<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.candidate.initialize_field(target, field, value)
    }

    pub fn write_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: OperationWrites<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        self.candidate.write_field(target, field, value)
    }

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
        Effect::PayloadBinding: ApplicationRetainedEffectBinding<Value = Payload>,
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

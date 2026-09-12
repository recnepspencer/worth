use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads, OperationUnlinks,
    WritePosture,
};

use super::super::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryPreserveOutput,
    WorthQueryRetireOutput,
};
use super::invariant::HandlerInterruption;

/// Borrow of the already-reserved candidate builder for one exact binding.
pub struct CandidateWriter<'borrow, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    candidate: &'borrow mut WorthQueryApplicationEffectProgramBuilder<
        Schema,
        Binding::Operation,
        Binding::Input,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
}

impl<'borrow, Schema, Binding> CandidateWriter<'borrow, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph) fn new(
        candidate: &'borrow mut WorthQueryApplicationEffectProgramBuilder<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    ) -> Self {
        Self { candidate }
    }

    pub fn candidate(
        &mut self,
    ) -> &mut WorthQueryApplicationEffectProgramBuilder<
        Schema,
        Binding::Operation,
        Binding::Input,
        <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    > {
        self.candidate
    }

    pub fn checkpoint(&self) -> Result<(), HandlerInterruption> {
        self.candidate
            .handler_checkpoint()
            .map_err(HandlerInterruption::from)
    }

    /// Resolves an existing effect target only from the completed decision read set.
    pub fn resolve_entity<Entity, Aspect, Field, Value, Write, Unit>(
        &self,
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
        value: Value,
    ) -> Result<
        WorthQueryApplicationEffectEntity<Schema, Entity>,
        super::super::WorthQueryApplicationAttemptDenial,
    >
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.candidate.resolve_observed_entity(field, value)
    }

    /// Removes the observed edge set between two targets using this attempt's
    /// completed decision facts. No runtime read occurs during authoring.
    pub fn unlink<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), super::super::WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationReads<Binding::Operation> + OperationUnlinks<Binding::Operation>,
    {
        self.candidate.unlink_observed(relation, from, to)
    }

    pub fn preserve_output<Entity>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, WorthQueryPreserveOutput>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), super::super::WorthQueryApplicationAttemptDenial>
    where
        Entity: 'static,
    {
        self.candidate.bind_output(role, target)
    }

    pub fn create_output<Entity>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, WorthQueryCreateOutput>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), super::super::WorthQueryApplicationAttemptDenial>
    where
        Entity: 'static,
    {
        self.candidate.bind_output(role, target)
    }

    pub fn retire_output<Entity>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, WorthQueryRetireOutput>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), super::super::WorthQueryApplicationAttemptDenial>
    where
        Entity: 'static,
    {
        self.candidate.bind_output(role, target)
    }
}

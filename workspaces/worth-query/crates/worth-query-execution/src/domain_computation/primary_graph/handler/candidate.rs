use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding, WorthQueryApplicationOutputAction,
    WorthQueryApplicationOutputRole, WorthQueryApplicationOutputRoleFamily, WorthQueryCreateOutput,
    WorthQueryPreserveOutput, WorthQueryRetireOutput,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads, OperationUnlinks,
    WritePosture,
};

use super::super::application_attempt::OutputRoleUse;
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryInvariantMutationTarget,
};
use super::invariant::HandlerInterruption;

mod verbs;

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
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.candidate.resolve_observed_entity(field, value)
    }

    /// Recover an effect target admitted from this attempt's completed typed
    /// decision read set without requiring a duplicate scalar identity field.
    pub fn projected_entity<Entity>(
        &self,
        target: &WorthQueryInvariantMutationTarget<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    {
        self.candidate.projected_entity(target)
    }

    /// Removes the observed edge set between two targets using this attempt's
    /// completed decision facts. No runtime read occurs during authoring.
    pub fn unlink<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationReads<Binding::Operation> + OperationUnlinks<Binding::Operation>,
    {
        self.candidate.unlink_observed(relation, from, to)
    }

    /// Bind `target` under the fixed role `Role` of this binding's output
    /// contract, keeping it. A role the contract does not declare with this
    /// entity, posture and cardinality fails to compile.
    pub fn preserve_output<Role>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
    {
        self.candidate
            .bind_output(OutputRoleUse::fixed::<Role>(), target)
    }

    /// Bind the created `target` under the fixed role `Role`.
    pub fn create_output<Role>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryCreateOutput,
        >,
    {
        self.candidate
            .bind_output(OutputRoleUse::fixed::<Role>(), target)
    }

    /// Bind the retired `target` under the fixed role `Role`.
    pub fn retire_output<Role>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryRetireOutput,
        >,
    {
        self.candidate
            .bind_output(OutputRoleUse::fixed::<Role>(), target)
    }

    /// Bind the kept `target` as the member of `Family` named by `suffix`. A
    /// family the contract does not declare, or one whose postures exclude
    /// preserve, fails to compile.
    pub fn preserve_member<Family>(
        &mut self,
        suffix: &str,
        target: &WorthQueryApplicationEffectEntity<Schema, Family::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Family: WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = Binding::Output>,
    {
        self.bind_member::<Family, WorthQueryPreserveOutput>(suffix, target)
    }

    /// Bind the created `target` as the member of `Family` named by `suffix`.
    pub fn create_member<Family>(
        &mut self,
        suffix: &str,
        target: &WorthQueryApplicationEffectEntity<Schema, Family::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Family: WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = Binding::Output>,
    {
        self.bind_member::<Family, WorthQueryCreateOutput>(suffix, target)
    }

    /// Bind the retired `target` as the member of `Family` named by `suffix`.
    pub fn retire_member<Family>(
        &mut self,
        suffix: &str,
        target: &WorthQueryApplicationEffectEntity<Schema, Family::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Family: WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = Binding::Output>,
    {
        self.bind_member::<Family, WorthQueryRetireOutput>(suffix, target)
    }

    fn bind_member<Family, Action>(
        &mut self,
        suffix: &str,
        target: &WorthQueryApplicationEffectEntity<Schema, Family::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Family: WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = Binding::Output>,
        Action: WorthQueryApplicationOutputAction,
    {
        let role = OutputRoleUse::member::<Family, Action>(suffix).map_err(|_| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::InvalidOutputRole,
                format!("{}{suffix}", Family::PREFIX),
            )
        })?;
        self.candidate.bind_output(role, target)
    }
}

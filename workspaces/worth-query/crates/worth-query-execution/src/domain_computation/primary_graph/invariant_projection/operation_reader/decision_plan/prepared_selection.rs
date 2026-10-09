use std::marker::PhantomData;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationOperationDecisionReadTarget,
    ApplicationScalarValueBinding, ApplicationSchema, ApplicationSchemaBindingIdentity,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads, WritePosture,
};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind,
    BoundedIndexParityMode, PreparedEntityFieldLookup,
};

use super::indexed_selection::selection_denial;
use super::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantDecisionPlanDenialKind,
};
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryEntityResolutionDenialKind, WorthQueryInvariantEntityIdentity,
};

/// A declared equality target prepared at one actual projection and operation
/// admission. It grants no mutation or latest-head authority and stores no policy.
pub struct WorthQueryPreparedEntitySelection<
    Schema,
    Operation,
    Entity,
    Aspect,
    Field,
    Value,
    Write,
    Unit,
> {
    field:
        ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, EqualityPredicate, Unit>,
    native: PreparedEntityFieldLookup,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    authority_identity: u64,
    binding_identity: ApplicationSchemaBindingIdentity,
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    admission_identity: Option<WorthQueryOperationAdmissionIdentity>,
    _operation: PhantomData<fn() -> Operation>,
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub fn prepare_entity_selection<Entity, Aspect, Field, Value, Write, Unit>(
        &mut self,
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
    ) -> Result<
        WorthQueryPreparedEntitySelection<
            Schema,
            Operation,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Unit,
        >,
        HandlerExecutionDenial,
    >
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader
            .retention_control
            .check_live()
            .map_err(HandlerExecutionDenial::new)?;
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Field {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })
        .map_err(HandlerExecutionDenial::new)?;
        let layout = self
            .reader
            .layout
            .equality_field(field.entity(), field.aspect(), field.field())
            .ok_or_else(|| {
                selection_denial(
                    WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
                    field.field(),
                )
            })?;
        let index = layout.equality_index_id.ok_or_else(|| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                field.field(),
            )
        })?;
        let view = self
            .reader
            .runtime
            .read_truth()
            .project_observation(&self.reader.observation)
            .map_err(|_| {
                selection_denial(
                    WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                    field.field(),
                )
            })?;
        let native = self
            .reader
            .runtime
            .index_access()
            .prepare_entity_field_lookup(&view, index, layout.entity_kind, &layout.locator)
            .map_err(|_| {
                selection_denial(
                    WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                    field.field(),
                )
            })?;
        self.reader
            .retention_control
            .check_live()
            .map_err(HandlerExecutionDenial::new)?;
        Ok(WorthQueryPreparedEntitySelection {
            field,
            native,
            snapshot: self.reader.snapshot.clone(),
            authority_identity: self.reader.authority_identity,
            binding_identity: self.binding_identity.clone(),
            runtime_authority: self.runtime_authority,
            admission_identity: self.admission_identity,
            _operation: PhantomData,
        })
    }

    pub fn decision_select_entities_prepared<Entity, Aspect, Field, Value, Write, Unit>(
        &mut self,
        prepared: &WorthQueryPreparedEntitySelection<
            Schema,
            Operation,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Unit,
        >,
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        if prepared.authority_identity != self.reader.authority_identity
            || prepared.snapshot != *self.reader.snapshot
            || prepared.snapshot.runtime_instance_id() != self.reader.runtime.runtime_instance_id()
            || prepared.runtime_authority != self.runtime_authority
            || prepared.binding_identity != *self.binding_identity
            || prepared.admission_identity != self.admission_identity
        {
            return Err(HandlerExecutionDenial::new(
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                    prepared.field.field(),
                ),
            ));
        }
        let control = self.reader.retention_control;
        control.check_live().map_err(HandlerExecutionDenial::new)?;
        self.require_selection_budget(candidate_limit, prepared.field.field())?;
        let value = Field::Binding::encode(&value).map_err(|_| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::ValueEncodingRejected,
                prepared.field.field(),
            )
        })?;
        let outcome = self
            .reader
            .runtime
            .index_access()
            .execute_prepared_entity_field_lookup(
                &prepared.native,
                &value,
                candidate_limit,
                BoundedIndexParityMode::Production,
                || control.check_live(),
            )
            .map_err(|stop| match stop {
                Stop::Admission(denial) => HandlerExecutionDenial::new(denial),
                Stop::Lookup(denial) => {
                    let examined = denial.examined_entry_count();
                    self.reader.work_budget.consume(1 + examined);
                    self.reader.work.record_lookup(examined);
                    let kind = match denial.kind() {
                        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
                        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
                            WorthQueryEntityResolutionDenialKind::CorruptIdentityIndex
                        }
                        _ => WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                    };
                    selection_denial(kind, prepared.field.field())
                }
                Stop::ExactBasisRequired | Stop::AccountingOverflow => selection_denial(
                    WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                    prepared.field.field(),
                ),
            })?;
        self.reader
            .work_budget
            .consume(1 + outcome.examined_entry_count());
        self.reader
            .work
            .record_lookup(outcome.examined_entry_count());
        if outcome.overflowed() {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::CandidateLimitExceeded {
                    maximum: candidate_limit,
                },
                prepared.field.field(),
            ));
        }
        self.retain_entity_selection(
            prepared.field,
            prepared.native.entity_kind(),
            prepared.native.field_locator().clone(),
            value,
            candidate_limit,
            outcome,
        )
    }
}

impl<Schema, Operation, Entity, Aspect, Field, Value, Write, Unit>
    WorthQueryPreparedEntitySelection<Schema, Operation, Entity, Aspect, Field, Value, Write, Unit>
{
    pub(in crate::domain_computation::primary_graph) fn field(
        &self,
    ) -> ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, EqualityPredicate, Unit>
    {
        self.field
    }
}

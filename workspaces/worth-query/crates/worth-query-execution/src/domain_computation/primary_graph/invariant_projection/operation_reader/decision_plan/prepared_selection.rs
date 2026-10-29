use super::super::super::WorthQueryInvariantProjectionDenial;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use std::marker::PhantomData;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationOperationDecisionReadTarget,
    ApplicationScalarValueBinding, ApplicationSchema, ApplicationSchemaBindingIdentity,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads, WritePosture,
};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind,
    BoundedIndexParityMode, PreparedEntityFieldLookup,
};

use super::indexed_selection::{selection_denial, typed_entity_selection};
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
        let native =
            self.prepare_named_entity_selection(field.entity(), field.aspect(), field.field())?;
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

    #[inline(never)]
    fn prepare_named_entity_selection(
        &mut self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<PreparedEntityFieldLookup, HandlerExecutionDenial> {
        let execution = self.reader.execution;
        self.reader
            .retention_control
            .check_live()
            .map_err(HandlerExecutionDenial::new)?;
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Field {
            entity: entity.to_owned(),
            aspect: aspect.to_owned(),
            field: field.to_owned(),
        })
        .map_err(HandlerExecutionDenial::new)?;
        let layout = self
            .reader
            .layout
            .equality_field(entity, aspect, field)
            .ok_or_else(|| {
                selection_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field,
                )
            })?;
        let index = layout.equality_index_id.ok_or_else(|| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::EqualityIndexUnavailable,
                ),
                field,
            )
        })?;
        let view = self
            .reader
            .runtime
            .read_truth()
            .project_observation(&self.reader.observation)
            .map_err(|_| {
                selection_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::EqualityIndexUnavailable,
                    ),
                    field,
                )
            })?;
        let native = self
            .reader
            .runtime
            .index_access()
            .prepare_entity_field_lookup(
                &view,
                index,
                layout.entity_kind,
                &layout.locator,
                execution,
            )
            .map_err(|denied| {
                if let BoundedEntityFieldLookupDenialKind::Execution(cause) = denied.kind() {
                    self.reader.native_read_stop.get_or_insert(cause);
                    return HandlerExecutionDenial::new(
                        WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None),
                    );
                }
                selection_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::EqualityIndexUnavailable,
                    ),
                    field,
                )
            })?;
        self.reader
            .retention_control
            .check_live()
            .map_err(HandlerExecutionDenial::new)?;
        Ok(native)
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
        let selected = self.select_prepared_named_entities(
            &prepared.native,
            &prepared.snapshot,
            prepared.authority_identity,
            &prepared.runtime_authority,
            &prepared.binding_identity,
            &prepared.admission_identity,
            prepared.field.entity(),
            prepared.field.field(),
            &mut || Field::Binding::encode(&value).map_err(|_| ()),
            candidate_limit,
        )?;
        Ok(typed_entity_selection(selected))
    }

    /// A foreign prepared identity is refused before the budget check and before encoding.
    #[inline(never)]
    fn select_prepared_named_entities(
        &mut self,
        native: &PreparedEntityFieldLookup,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        authority_identity: u64,
        runtime_authority: &crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
        admission_identity: &Option<WorthQueryOperationAdmissionIdentity>,
        entity: &str,
        field: &str,
        encode: &mut dyn FnMut() -> Result<worth_foundational::facade::AspectValue, ()>,
        candidate_limit: usize,
    ) -> Result<super::indexed_selection::RetainedEntitySelection, HandlerExecutionDenial> {
        let execution = self.reader.execution;
        if authority_identity != self.reader.authority_identity
            || snapshot != self.reader.snapshot
            || snapshot.runtime_instance_id() != self.reader.runtime.runtime_instance_id()
            || runtime_authority != &self.runtime_authority
            || binding_identity != self.binding_identity
            || admission_identity != &self.admission_identity
        {
            return Err(HandlerExecutionDenial::new(
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                    field,
                ),
            ));
        }
        let control = self.reader.retention_control;
        control.check_live().map_err(HandlerExecutionDenial::new)?;
        self.require_selection_budget(candidate_limit, field)?;
        let value = encode().map_err(|_| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field,
            )
        })?;
        let outcome = self
            .reader
            .runtime
            .index_access()
            .execute_prepared_entity_field_lookup(
                native,
                &value,
                candidate_limit,
                BoundedIndexParityMode::Production,
                execution,
                || control.check_live(),
            )
            .map_err(|stop| match stop {
                Stop::Admission(denial) => HandlerExecutionDenial::new(denial),
                Stop::Lookup(denial) => {
                    let examined = denial.examined_entry_count();
                    let admitted = self.reader.work_budget.consume(1 + examined);
                    if admitted {
                        self.reader.work.record_lookup(examined);
                    }
                    let kind = match denial.kind() {
                        BoundedEntityFieldLookupDenialKind::Execution(cause) => {
                            self.reader.native_read_stop.get_or_insert(cause);
                            return HandlerExecutionDenial::new(
                                WorthQueryInvariantProjectionDenial::execution_request_stopped(
                                    cause, None,
                                ),
                            );
                        }
                        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
                        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
                            WorthQueryEntityResolutionDenialKind::Refused(
                                ResolutionRefusal::CorruptIdentityIndex,
                            )
                        }
                        BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
                        | BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
                        | BoundedEntityFieldLookupDenialKind::IndexNotInstalled
                        | BoundedEntityFieldLookupDenialKind::WrongIndexKind
                        | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable => {
                            WorthQueryEntityResolutionDenialKind::Refused(
                                ResolutionRefusal::EqualityIndexUnavailable,
                            )
                        }
                    };
                    if !admitted {
                        return selection_denial(
                            WorthQueryEntityResolutionDenialKind::Stopped(
                                NativeReadStop::WorkExhausted,
                            ),
                            field,
                        );
                    }
                    selection_denial(kind, field)
                }
                Stop::AccountingOverflow => {
                    let cause = NativeReadStop::WorkCounterOverflow;
                    self.reader.native_read_stop.get_or_insert(cause);
                    HandlerExecutionDenial::new(
                        WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, None),
                    )
                }
                Stop::ExactBasisRequired => selection_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::EqualityIndexUnavailable,
                    ),
                    field,
                ),
            })?;
        if !self
            .reader
            .work_budget
            .consume(1 + outcome.examined_entry_count())
        {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field,
            ));
        }
        self.reader
            .work
            .record_lookup(outcome.examined_entry_count());
        if outcome.overflowed() {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::CandidateLimitExceeded {
                        maximum: candidate_limit,
                    },
                ),
                field,
            ));
        }
        self.retain_named_entity_selection(
            entity,
            native.entity_kind(),
            native.field_locator().clone(),
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

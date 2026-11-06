use super::super::super::WorthQueryInvariantProjectionDenial;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use std::marker::PhantomData;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationScalarValueBinding, ApplicationSchema,
    ApplicationSchemaBindingIdentity,
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
    Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
> {
    field: Field,
    native: PreparedEntityFieldLookup,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    phase: Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    authority_identity: u64,
    binding_identity: ApplicationSchemaBindingIdentity,
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    admission_identity: Option<WorthQueryOperationAdmissionIdentity>,
    _operation: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub fn prepare_entity_selection<Entity, Aspect, Field, Value>(
        &mut self,

        field: Field,
    ) -> Result<WorthQueryPreparedEntitySelection<Schema, Operation, Field>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        self.reader
            .layout
            .record_selection(field)
            .map_err(HandlerExecutionDenial::new)?;
        let execution = self.reader.execution;
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
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::FieldNotInstalled,
                    ),
                    field.field(),
                )
            })?;
        let index = layout.equality_index_id.ok_or_else(|| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::EqualityIndexUnavailable,
                ),
                field.field(),
            )
        })?;
        let native = match &self.reader.native {
            super::super::super::read_basis::NativeProjectionRead::Committed(runtime) => {
                let view = runtime
                    .read_truth()
                    .project_observation(&self.reader.observation)
                    .map_err(|_| {
                        selection_denial(
                            WorthQueryEntityResolutionDenialKind::Refused(
                                ResolutionRefusal::EqualityIndexUnavailable,
                            ),
                            field.field(),
                        )
                    })?;
                runtime.index_access().prepare_entity_field_lookup(
                    &view,
                    index,
                    layout.entity_kind,
                    &layout.locator,
                    execution,
                )
            }
            super::super::super::read_basis::NativeProjectionRead::Applied { before, .. } => before
                .prepare_entity_field_lookup(index, layout.entity_kind, &layout.locator, execution),
        }
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
            phase: self
                .reader
                .native
                .candidate_identity()
                .map(|id| id.qualification()),
            authority_identity: self.reader.authority_identity,
            binding_identity: self.binding_identity.clone(),
            runtime_authority: self.runtime_authority,
            admission_identity: self.admission_identity,
            _operation: PhantomData,
        })
    }

    pub fn decision_select_entities_prepared<Entity, Aspect, Field, Value>(
        &mut self,

        prepared: &WorthQueryPreparedEntitySelection<Schema, Operation, Field>,
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let execution = self.reader.execution;
        if prepared.phase
            != self
                .reader
                .native
                .candidate_identity()
                .map(|id| id.qualification())
            || prepared.authority_identity != self.reader.authority_identity
            || prepared.snapshot != *self.reader.snapshot
            || prepared.snapshot.runtime_instance_id() != self.reader.native.runtime_instance_id()
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
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                prepared.field.field(),
            )
        })?;
        let outcome = match &self.reader.native {
            super::super::super::read_basis::NativeProjectionRead::Committed(runtime) => {
                let outcome =
                    runtime
                        .index_access()
                        .execute_prepared_entity_field_lookup(
                            &prepared.native,
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
                                        prepared.field.field(),
                                    );
                                }
                                selection_denial(kind, prepared.field.field())
                            }
                            Stop::AccountingOverflow => {
                                let cause = NativeReadStop::WorkCounterOverflow;
                                self.reader.native_read_stop.get_or_insert(cause);
                                HandlerExecutionDenial::new(
                                    WorthQueryInvariantProjectionDenial::execution_request_stopped(
                                        cause, None,
                                    ),
                                )
                            }
                            Stop::ExactBasisRequired => selection_denial(
                                WorthQueryEntityResolutionDenialKind::Refused(
                                    ResolutionRefusal::EqualityIndexUnavailable,
                                ),
                                prepared.field.field(),
                            ),
                        })?;
                super::super::super::read_basis::ProjectionEntitySelection::committed(outcome)
            }
            super::super::super::read_basis::NativeProjectionRead::Applied {
                candidate, ..
            } => super::super::super::read_basis::ProjectionEntitySelection::applied(
                candidate,
                &prepared.native,
                &value,
                candidate_limit,
                execution,
            )
            .map_err(|cause| {
                let examined = cause.examined_entry_count();
                self.reader.work.record_lookup(examined);
                let kind = match cause.kind() {
                    BoundedEntityFieldLookupDenialKind::Execution(cause) => {
                        WorthQueryEntityResolutionDenialKind::Stopped(cause)
                    }
                    BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
                    | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
                        WorthQueryEntityResolutionDenialKind::Refused(
                            ResolutionRefusal::CorruptIdentityIndex,
                        )
                    }
                    _ => WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::EqualityIndexUnavailable,
                    ),
                };
                if !self.reader.work_budget.consume(1 + examined) {
                    return selection_denial(
                        WorthQueryEntityResolutionDenialKind::Stopped(
                            NativeReadStop::WorkExhausted,
                        ),
                        prepared.field.field(),
                    );
                }
                selection_denial(kind, prepared.field.field())
            })?,
        };
        control.check_live().map_err(HandlerExecutionDenial::new)?;
        if !self
            .reader
            .work_budget
            .consume(1 + outcome.examined_entry_count())
        {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                prepared.field.field(),
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
                prepared.field.field(),
            ));
        }
        self.retain_named_entity_selection(
            prepared.field.entity(),
            prepared.native.entity_kind(),
            prepared.native.field_locator().clone(),
            value,
            candidate_limit,
            outcome,
        )
    }
}

impl<
        Schema,
        Operation,
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
    > WorthQueryPreparedEntitySelection<Schema, Operation, Field>
{
    pub(in crate::domain_computation::primary_graph) fn field(&self) -> Field {
        self.field
    }
}

use super::super::super::WorthQueryInvariantProjectionDenial;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use std::marker::PhantomData;
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as NativeReadStop;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationOperationDecisionReadTarget,
    ApplicationScalarValueBinding, ApplicationSchema, DeclaredApplicationFieldValue,
    EqualityPredicate, OperationReads, WritePosture,
};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact, HandlerExecutionDenial,
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryInvariantEntityIdentity,
};

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Select the complete equality result, retaining presence and absence for
    /// commit comparison. Overflow denies rather than returning a truncated set.
    pub fn decision_select_entities<Entity, Aspect, Field, Value, Write, Unit>(
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
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let execution = self.reader.execution;
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Field {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })
        .map_err(HandlerExecutionDenial::new)?;
        self.require_selection_budget(candidate_limit, field.field())?;
        let value = Field::Binding::encode(&value).map_err(|_| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            )
        })?;
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
        let index_id = layout.equality_index_id.ok_or_else(|| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::EqualityIndexUnavailable,
                ),
                field.field(),
            )
        })?;
        let request = BoundedEntityFieldLookupRequest::new(
            self.reader.snapshot.clone(),
            index_id,
            layout.entity_kind,
            layout.locator.clone(),
            value.clone(),
            candidate_limit,
        )
        .map_err(|_| {
            selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::InvalidCandidateLimit,
                ),
                field.field(),
            )
        })?;
        let outcome = self
            .reader
            .runtime
            .index_access()
            .execute_bounded_entity_field_lookup(
                request,
                BoundedIndexParityMode::Production,
                execution,
            )
            .map_err(|denial| {
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
                        field.field(),
                    );
                }
                selection_denial(kind, field.field())
            })?;
        if !self
            .reader
            .work_budget
            .consume(1 + outcome.examined_entry_count())
        {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                field.field(),
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
                field.field(),
            ));
        }
        self.retain_entity_selection(
            field,
            layout.entity_kind,
            layout.locator.clone(),
            value,
            candidate_limit,
            outcome,
        )
    }

    pub(super) fn retain_entity_selection<Entity, Aspect, Field, Value, Write, Unit>(
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
        kind: worth_relational::facade::identity::KindId,
        locator: worth_foundational::facade::AspectFieldLocator,
        value: worth_foundational::facade::AspectValue,
        candidate_limit: usize,
        outcome: worth_relational::facade::indexes::BoundedEntityFieldLookupOutcome,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Unit: ApplicationFieldUnit,
    {
        let index_id = outcome.retain_definition().index_id;
        let mut identities = Vec::with_capacity(outcome.candidate_entity_ids().len());
        for entity_id in outcome.candidate_entity_ids() {
            self.reader.realized_scope.record(*entity_id);
            identities.push(WorthQueryInvariantEntityIdentity {
                entity_id: *entity_id,
                kind,
                entity: Arc::from(field.entity()),
                authority_identity: self.reader.authority_identity,
                _marker: PhantomData,
            });
        }
        let fact = WorthQueryApplicationObservedFact::IndexedEntitySelection {
            index_id,
            definition: outcome.retain_definition(),
            entity_kind: kind,
            locator,
            value,
            candidate_limit,
            candidates: outcome.into_candidate_entity_ids(),
        };
        self.reader
            .dependent_source_facts
            .capture(fact, self.reader.retention_control)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(identities)
    }

    pub(super) fn require_selection_budget(
        &mut self,
        limit: usize,
        subject: &str,
    ) -> Result<(), HandlerExecutionDenial> {
        if limit == 0 || limit == usize::MAX {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::InvalidCandidateLimit,
                ),
                subject,
            ));
        }
        if !self.reader.work_budget.can_afford(1 + limit) {
            return Err(selection_denial(
                WorthQueryEntityResolutionDenialKind::Stopped(NativeReadStop::WorkExhausted),
                subject,
            ));
        }
        Ok(())
    }
}

pub(super) fn selection_denial(
    kind: WorthQueryEntityResolutionDenialKind,
    subject: &str,
) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryEntityResolutionDenial::new(kind, subject))
}

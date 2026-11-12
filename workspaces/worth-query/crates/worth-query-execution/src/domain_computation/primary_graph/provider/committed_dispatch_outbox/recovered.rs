//! Recovered correlation selects original artifacts; each read borrows the exact Native basis.
use super::*;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use std::collections::BTreeMap;
use worth_relational::facade::{
    durability::{RelationalAssociatedEffectRefusal, RelationalAssociatedEffectSubject},
    identity::EntityId,
};
use worth_runtime_world::facade::{
    RuntimeWorldInspectionPort, RuntimeWorldObservationPort, WorldReadmittedPublication,
};

pub(super) struct RecoveredDispatchCandidate {
    pub(super) version: worth_relational::facade::identity::VersionId,
    pub(super) entity: EntityId,
    pub(super) expected: WorthQueryDispatchOutboxRecord,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::provider) struct RecoveredDispatchSelector {
    subject: RelationalAssociatedEffectSubject,
    entity: EntityId,
    record: WorthQueryDispatchOutboxRecord,
}
impl RecoveredDispatchSelector {
    pub(in crate::domain_computation::primary_graph::provider) fn same_source(
        &self,
        other: &Self,
    ) -> bool {
        self.subject.receipt() == other.subject.receipt()
            && self.subject.position() == other.subject.position()
            && self.subject.artifact_digest() == other.subject.artifact_digest()
            && self.entity == other.entity
            && self.record == other.record
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn readmit_dispatch_correlation_index(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &RuntimeWorldObservationPort,
        inspection: &RuntimeWorldInspectionPort,
        candidates: Vec<RecoveredDispatchCandidate>,
    ) -> Result<(), Denial> {
        // A Home without outboxes needs no artifact discovery or source-body reads.
        if candidates.is_empty() {
            return self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .readmit_dispatch_correlations(BTreeMap::new())
                .map_err(|()| Denial::AmbiguousCorrelation);
        }
        let sources = self.graph.with_runtime(execution, |runtime| {
            let authority = runtime.durability_authority();
            let subjects = authority
                .discover_all_associated_effects(execution)
                .map_err(native_denial)?;
            let mut by_version = BTreeMap::new();
            for subject in subjects {
                if by_version
                    .insert(subject.receipt().version_id, subject)
                    .is_some()
                {
                    return Err(Denial::AmbiguousCorrelation);
                }
            }
            let mut sources = BTreeMap::new();
            for candidate in candidates {
                let subject = by_version
                    .get(&candidate.version)
                    .ok_or(Denial::NotAuthoritative)?;
                let native = authority
                    .readmit_associated_effect(execution, subject)
                    .map_err(native_denial)?;
                let original = observation
                    .readmitted_native_publication(execution, native)
                    .map_err(world_denial)?;
                let description = describe(&original, inspection)?;
                let record =
                    self.original_record(execution, &original, candidate.entity, &description)?;
                if record != candidate.expected {
                    return Err(Denial::RecordMismatch);
                }
                let selector = RecoveredDispatchSelector {
                    subject: subject.clone(),
                    entity: candidate.entity,
                    record,
                };
                self.read_recovered_with_original(
                    execution,
                    runtime,
                    &selector,
                    &original,
                    description,
                )?;
                if sources
                    .insert(*selector.record.correlation(), selector)
                    .is_some()
                {
                    return Err(Denial::AmbiguousCorrelation);
                }
            }
            Ok(sources)
        })?;
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .readmit_dispatch_correlations(sources)
            .map_err(|()| Denial::AmbiguousCorrelation)
    }

    pub(super) fn recovered_dispatch_for_correlation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlation: &ExternalEffectCorrelationIdentity,
        observation: &RuntimeWorldObservationPort,
        inspection: &RuntimeWorldInspectionPort,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let selector = self
            .completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .recovered_dispatch_correlation(correlation)
            .map_err(|()| Denial::AmbiguousCorrelation)?;
        let Some(selector) = selector else {
            return Ok(None);
        };
        self.read_recovered_dispatch_source(execution, &selector, observation, inspection)
            .map(|(owner, _source)| Some(owner))
    }

    pub(in crate::domain_computation::primary_graph::provider) fn read_recovered_dispatch_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selector: &RecoveredDispatchSelector,
        observation: &RuntimeWorldObservationPort,
        inspection: &RuntimeWorldInspectionPort,
    ) -> Result<
        (
            WorthQueryCommittedDispatchOutboxObservation,
            WorldReadmittedPublication,
        ),
        Denial,
    > {
        self.graph.with_runtime(execution, |runtime| {
            let native = runtime
                .durability_authority()
                .readmit_associated_effect(execution, &selector.subject)
                .map_err(native_denial)?;
            let original = observation
                .readmitted_native_publication(execution, native)
                .map_err(world_denial)?;
            let description = describe(&original, inspection)?;
            let record =
                self.original_record(execution, &original, selector.entity, &description)?;
            if record != selector.record {
                return Err(Denial::RecordMismatch);
            }
            self.read_recovered_with_original(execution, runtime, selector, &original, description)
                .map(|owner| (owner, original))
        })
    }

    fn original_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &WorldReadmittedPublication,
        entity: EntityId,
        description: &WorthQueryCommittedProductPublication,
    ) -> Result<WorthQueryDispatchOutboxRecord, Denial> {
        let row = original
            .created_entity(execution, entity)
            .map_err(native_denial)?
            .ok_or(Denial::Missing)?;
        let layout = self.graph.layout.provider_dispatch_outbox();
        if row.kind_id() != layout.entity_kind {
            return Err(Denial::WrongRecordKind);
        }
        let aspect = layout.correlation_locator.aspect().aspect_key();
        let fields = decode_fields(
            required_fields(layout)?
                .iter()
                .map(|key| row.field(aspect, key).cloned())
                .collect::<Option<Vec<_>>>()
                .ok_or(Denial::Malformed)?,
        )?;
        let idempotency = self.graph.layout.provider_idempotency();
        let mut binding = None;
        for created in original.semantic_result().created_entities() {
            if created.key.kind != idempotency.entity_kind {
                continue;
            }
            let row = original
                .created_entity(execution, created.assigned)
                .map_err(native_denial)?
                .ok_or(Denial::NotAuthoritative)?;
            let candidate = super::super::idempotency::snapshot_resolution::original_binding::read(
                &row,
                idempotency,
            )
            .map_err(idempotency_denial)?;
            if candidate.outcome.get() == fields.outcome_identity
                && binding.replace(candidate).is_some()
            {
                return Err(Denial::AmbiguousCorrelation);
            }
        }
        let binding = binding.ok_or(Denial::NotAuthoritative)?;
        let affinity =
            super::super::idempotency::WorthQueryProductIdempotencyAffinity::from_publication(
                description,
            );
        if binding.affinity != super::super::idempotency::recorded_origin::affinity_text(&affinity)
        {
            return Err(Denial::NotAuthoritative);
        }
        super::super::WorthQueryRecoveredDispatchOutbox::from_source_row(
            RecordRef::Entity(entity),
            fields,
            &binding.intent,
            &self.installed_operations,
        )
        .readmitted_record()
    }

    fn read_recovered_with_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        selector: &RecoveredDispatchSelector,
        original: &WorldReadmittedPublication,
        description: WorthQueryCommittedProductPublication,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        if original.native_receipt() != selector.subject.receipt() {
            return Err(Denial::CommitMismatch);
        }
        let lease = runtime
            .retain_component_basis(original.after().basis().relational_basis())
            .map_err(|_| Denial::ExactCommitUnavailable)?;
        let record_ref = RecordRef::Entity(selector.entity);
        let answer = CommittedOutboxRead {
            runtime,
            layout: self.graph.layout.provider_dispatch_outbox(),
            record: &selector.record,
            record_ref: &record_ref,
            expected_commit: original.native_receipt(),
            expected_runtime: runtime.runtime_instance_id(),
            retained_basis: &lease,
        }
        .resolve(execution);
        runtime
            .release_component_basis(lease)
            .expect("the exact recovered read releases its same-owner Native loan");
        answer.map(|owner| WorthQueryCommittedDispatchOutboxObservation::seal(owner, description))
    }
}

fn describe(
    original: &WorldReadmittedPublication,
    inspection: &RuntimeWorldInspectionPort,
) -> Result<WorthQueryCommittedProductPublication, Denial> {
    let history = inspection
        .trace_ancestry(
            original.after().selected_commit().clone(),
            std::num::NonZeroUsize::new(1).unwrap(),
        )
        .map_err(|_| Denial::NotAuthoritative)?;
    let commit = history.commits().next().ok_or(Denial::NotAuthoritative)?;
    WorthQueryCommittedProductPublication::from_readmitted(original, commit)
}

pub(in crate::domain_computation::primary_graph::provider) fn idempotency_denial(
    cause: super::super::WorthQueryProviderIdempotencyResolutionDenial,
) -> Denial {
    use super::super::WorthQueryProviderIdempotencyResolutionDenial as Source;
    match cause {
        Source::ExecutionDenied(cause) => Denial::ReadExecution(cause),
        Source::ForeignAffinity
        | Source::PreparationBusy
        | Source::ActiveSnapshotCapacityExhausted { .. }
        | Source::RetentionCapacityExhausted
        | Source::RetentionIdentityExhausted
        | Source::SnapshotIdentityExhausted
        | Source::RecordedIntentUnverifiable
        | Source::Unavailable
        | Source::NativeKeyLookup { .. } => Denial::NotAuthoritative,
    }
}

pub(in crate::domain_computation::primary_graph::provider) fn native_denial(
    cause: RelationalAssociatedEffectRefusal,
) -> Denial {
    idempotency_denial(super::super::idempotency::snapshot_resolution::source_denial::native(cause))
}
pub(in crate::domain_computation::primary_graph::provider) fn world_denial(
    cause: worth_runtime_world::facade::RuntimeWorldServiceDenial<
        worth_runtime_world::facade::WorldReadmittedPublicationDenial,
    >,
) -> Denial {
    idempotency_denial(super::super::idempotency::snapshot_resolution::source_denial::world(cause))
}

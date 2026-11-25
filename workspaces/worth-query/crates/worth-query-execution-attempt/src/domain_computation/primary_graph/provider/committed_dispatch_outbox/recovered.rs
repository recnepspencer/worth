//! Recovered correlation selects original artifacts; each read borrows the exact Native basis.
use super::*;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use std::collections::BTreeMap;
use worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_relational::facade::{
    durability::{RelationalAssociatedEffectRefusal, RelationalAssociatedEffectSubject},
    identity::EntityId,
};
use worth_runtime_world::facade::{
    RuntimeWorldInspectionPort, RuntimeWorldObservationPort, WorldObservedHomePublication,
};

pub(super) struct RecoveredDispatchCandidate {
    pub(super) version: worth_relational::facade::identity::VersionId,
    pub(super) entity: EntityId,
    pub(super) expected: WorthQueryDispatchOutboxRecord,
}

pub use super::pending_recovered::PendingRecoveredDispatch;

#[derive(Clone)]
pub struct RecoveredDispatchSelector {
    pub(super) subject: RelationalAssociatedEffectSubject,
    pub(super) entity: EntityId,
    pub(super) record: WorthQueryDispatchOutboxRecord,
}
impl RecoveredDispatchSelector {
    pub fn same_source(&self, other: &Self) -> bool {
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
        unpublished: (
            &BTreeMap<
                worth_relational::facade::history::CommitId,
                worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
            >,
            &std::collections::BTreeSet<worth_relational::facade::history::CommitId>,
        ),
    ) -> Result<(), Denial> {
        let (continuable_native, unpublished_native) = unpublished;
        // A Home without outboxes needs no artifact discovery or source-body reads.
        if candidates.is_empty() {
            return self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .readmit_dispatch_correlations(BTreeMap::new(), BTreeMap::new())
                .map_err(|()| Denial::AmbiguousCorrelation);
        }
        let (sources, pending) = self.graph.with_runtime(execution, |runtime| {
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
            let mut pending = BTreeMap::new();
            for candidate in candidates {
                let subject = by_version
                    .get(&candidate.version)
                    .ok_or(Denial::NotAuthoritative)?;
                let correlation = *candidate.expected.correlation();
                if let Some(&attempt) = continuable_native.get(&subject.receipt().commit_id) {
                    let entry = PendingRecoveredDispatch {
                        selector: RecoveredDispatchSelector {
                            subject: subject.clone(),
                            entity: candidate.entity,
                            record: candidate.expected,
                        },
                        attempt,
                    };
                    if sources.contains_key(&correlation)
                        || pending.insert(correlation, entry).is_some()
                    {
                        return Err(Denial::AmbiguousCorrelation);
                    }
                    continue;
                }
                if unpublished_native.contains(&subject.receipt().commit_id) {
                    // Released original: terminal and can never publish.
                    continue;
                }
                let native = authority
                    .readmit_associated_effect(execution, subject)
                    .map_err(native_denial)?;
                let original = observation
                    .observe_home_original_publication(execution, native)
                    .map_err(world_denial)?;
                let description = describe_observed(&original, inspection)?;
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
                self.read_recovered_with_original(runtime, &selector, &original, description)?;
                if pending.contains_key(&correlation)
                    || sources.insert(correlation, selector).is_some()
                {
                    return Err(Denial::AmbiguousCorrelation);
                }
            }
            Ok((sources, pending))
        })?;
        self.completed_commit_evidence
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .readmit_dispatch_correlations(sources, pending)
            .map_err(|()| Denial::AmbiguousCorrelation)
    }

    pub(super) fn recovered_dispatch_for_correlation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlation: &ExternalEffectCorrelationIdentity,
        observation: &RuntimeWorldObservationPort,
        inspection: &RuntimeWorldInspectionPort,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let (selector, pending) = {
            let store = self
                .completed_commit_evidence
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (
                store
                    .recovered_dispatch_correlation(correlation)
                    .map_err(|()| Denial::AmbiguousCorrelation)?,
                store
                    .pending_recovered_dispatch_correlation(correlation)
                    .map_err(|()| Denial::AmbiguousCorrelation)?,
            )
        };
        if let Some(selector) = selector {
            return self
                .read_recovered_dispatch_source(execution, &selector, observation, inspection)
                .map(|(owner, _source)| Some(owner));
        }
        let Some(pending) = pending else {
            return Ok(None);
        };
        self.resolve_pending_recovered_dispatch(
            execution,
            correlation,
            pending,
            (observation, inspection),
        )
    }

    pub fn read_recovered_dispatch_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selector: &RecoveredDispatchSelector,
        observation: &RuntimeWorldObservationPort,
        inspection: &RuntimeWorldInspectionPort,
    ) -> Result<
        (
            WorthQueryCommittedDispatchOutboxObservation,
            WorldObservedHomePublication,
        ),
        Denial,
    > {
        self.graph.with_runtime(execution, |runtime| {
            let native = runtime
                .durability_authority()
                .readmit_associated_effect(execution, &selector.subject)
                .map_err(native_denial)?;
            let original = observation
                .observe_home_original_publication(execution, native)
                .map_err(world_denial)?;
            let description = describe_observed(&original, inspection)?;
            let record =
                self.original_record(execution, &original, selector.entity, &description)?;
            if record != selector.record {
                return Err(Denial::RecordMismatch);
            }
            self.read_recovered_with_original(runtime, selector, &original, description)
                .map(|owner| (owner, original))
        })
    }

    pub(super) fn original_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &WorldObservedHomePublication,
        entity: EntityId,
        description: &WorthQueryCommittedProductPublication,
    ) -> Result<WorthQueryDispatchOutboxRecord, Denial> {
        let row = original
            .created_entity(execution, entity)
            .map_err(native_denial)?
            .ok_or(Denial::Missing)?;
        let layout = self.graph.layout_ref().provider_dispatch_outbox();
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
        let idempotency = self.graph.layout_ref().provider_idempotency();
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

    pub(super) fn read_recovered_with_original(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        selector: &RecoveredDispatchSelector,
        original: &WorldObservedHomePublication,
        description: WorthQueryCommittedProductPublication,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        if original.native_receipt() != selector.subject.receipt() {
            return Err(Denial::CommitMismatch);
        }
        let committed = runtime
            .history()
            .immutable_commit_receipt_for_version(original.native_receipt().version_id)
            .ok_or(Denial::NotAuthoritative)?;
        if &committed != original.native_receipt() {
            return Err(Denial::CommitMismatch);
        }
        let owner = WorthQueryCommittedDispatchOutboxOwnerObservation::seal(
            selector.record.clone(),
            committed,
            RecordRef::Entity(selector.entity),
            runtime.runtime_instance_id(),
            WorthQueryCommittedDispatchOutboxReadWork::from_recovered_source(),
        );
        Ok(WorthQueryCommittedDispatchOutboxObservation::seal(
            owner,
            description,
        ))
    }
}

pub(super) fn describe_observed(
    original: &WorldObservedHomePublication,
    inspection: &RuntimeWorldInspectionPort,
) -> Result<WorthQueryCommittedProductPublication, Denial> {
    use worth_runtime_world::facade::{
        CompositeComponentChangePosture as Posture, CompositeHistoryCatalogDenial as HistoryDenial,
        RuntimeWorldServiceDenial as ServiceDenial,
    };
    let signal_publication = match original.signal_posture() {
        Posture::RetainExact => None,
        Posture::Published => match inspection.trace_ancestry(
            original.composite_commit().clone(),
            std::num::NonZeroUsize::MIN,
        ) {
            Ok(history) => {
                let commit = history.commits().next().ok_or(Denial::CommitMismatch)?;
                if commit.signal_change() != Posture::Published {
                    return Err(Denial::CommitMismatch);
                }
                let signal = commit
                    .signal_publication_identity()
                    .cloned()
                    .ok_or(Denial::CommitMismatch)?;
                Some(signal)
            }
            Err(ServiceDenial::Denied(HistoryDenial::UnknownProtectionTarget(_))) => None,
            Err(ServiceDenial::Denied(HistoryDenial::Cancelled)) => {
                return Err(Denial::ReadExecution(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
                )));
            }
            Err(ServiceDenial::Denied(HistoryDenial::DeadlineElapsed)) => {
                return Err(Denial::ReadExecution(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
                )));
            }
            Err(_) => return Err(Denial::NotAuthoritative),
        },
    };
    WorthQueryCommittedProductPublication::from_observed_home(original, signal_publication)
}


pub fn idempotency_denial(
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

pub fn native_denial(cause: RelationalAssociatedEffectRefusal) -> Denial {
    idempotency_denial(super::super::idempotency::snapshot_resolution::source_denial::native(cause))
}
pub fn world_denial(
    cause: worth_runtime_world::facade::RuntimeWorldServiceDenial<
        worth_runtime_world::facade::WorldReadmittedPublicationDenial,
    >,
) -> Denial {
    idempotency_denial(super::super::idempotency::snapshot_resolution::source_denial::world(cause))
}

use worth_proof::TransitionOutcome;

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::{
        AdmittedPhysicalMutation, PhysicalMutationIdempotencyRegistryAdmission,
        PhysicalMutationIdempotencyRegistryAdmissionError,
        PhysicalMutationIdempotencyRegistryDenial, PhysicalMutationOperationFamily,
    },
    record_serving::{
        planning::batch_placement::preflight_placement,
        publication::{
            prepare_canonical_payload,
            CanonicalPayloadPreparationError, CanonicalRecordAppendPayload,
            PhysicalMutationPreparationDeferred, PhysicalMutationPreparationDenial,
            PhysicalMutationPreparationFailure, PhysicalMutationPreparationOutcome,
            PhysicalMutationPreparationRebindRequired, PhysicalMutationPreparationStale,
            PhysicalMutationPreparationSuccess, PhysicalMutationResourceShape,
            PreparedPhysicalMutation, PreparedPhysicalMutationContext,
        },
        AdmittedRecordPlacementPolicy, RecordAppendBatch, RecordAppendDenial, RecordAppendError,
    },
    work::PhysicalMutationIdentityReservationError,
    PhysicalGroupQueueAdmissionTick, PhysicalMutationDeadline, PhysicalMutationIdempotencyKey,
    PhysicalMutationRequest, PhysicalMutationRequestFingerprint, PhysicalWorkSubmissionFailure,
    PhysicalWorkSubmissionStale,
};

pub(super) struct AdmittedMutationPreparation {
    pub(super) admission: AdmittedPhysicalMutation,
    pub(super) deadline: PhysicalMutationDeadline,
}

pub(super) enum PhysicalMutationPreparationAdmission {
    Prepared(AdmittedMutationPreparation),
    Completed(crate::physical_runtime::CompletedPhysicalMutation),
    ProvenNoEffect(crate::physical_runtime::ProvenNoEffectPhysicalMutation),
    Indeterminate(crate::physical_runtime::IndeterminatePhysicalMutation),
}

impl RecordPublicationDirector {
    pub(super) fn prepare_durable_append(
        &self,
        batch: RecordAppendBatch,
        placement: AdmittedRecordPlacementPolicy,
        manifest_capacity_transition: crate::physical_runtime::PhysicalManifestCapacityTransition,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        if let Err(outcome) = self.require_preparation_health() {
            return outcome;
        }
        if let Err(outcome) =
            self.preflight_durable_append(&batch, placement, manifest_capacity_transition)
        {
            return outcome;
        }
        let payload = match canonical_payload(batch) {
            Ok(payload) => payload,
            Err(outcome) => return outcome,
        };
        let group_queue_admission = match self.group_queue_admission_tick() {
            Ok(tick) => tick,
            Err(outcome) => return outcome,
        };
        let admitted = match self.admit_mutation_preparation(
            placement,
            manifest_capacity_transition,
            payload.digest,
            request,
            PhysicalMutationOperationFamily::RecordAppend,
        ) {
            Ok(admitted) => admitted,
            Err(outcome) => return outcome,
        };
        let admitted = match admitted {
            PhysicalMutationPreparationAdmission::Prepared(admitted) => admitted,
            PhysicalMutationPreparationAdmission::ProvenNoEffect(terminal) => {
                return TransitionOutcome::success(
                    PhysicalMutationPreparationSuccess::ProvenNoEffect(terminal),
                )
                .into()
            }
            PhysicalMutationPreparationAdmission::Completed(terminal) => {
                return TransitionOutcome::success(PhysicalMutationPreparationSuccess::Completed(
                    terminal,
                ))
                .into()
            }
            PhysicalMutationPreparationAdmission::Indeterminate(terminal) => {
                return TransitionOutcome::success(
                    PhysicalMutationPreparationSuccess::Indeterminate(terminal),
                )
                .into()
            }
        };
        let resources =
            PhysicalMutationResourceShape::prepared(payload.record_count, payload.payload_bytes);
        TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(
            PreparedPhysicalMutation::new(
                admitted.admission,
                payload.batch,
                payload.materialization,
                PreparedPhysicalMutationContext {
                    placement,
                    manifest_capacity_transition,
                    deadline: admitted.deadline,
                    group_queue_admission,
                    signal_profile: self.signal_profile,
                    durability_policy_basis: self.durability_policy_basis.clone(),
                    resources,
                    start: crate::physical_runtime::PhysicalMutationRuntimeOwner::start_port(
                        &self.mutations,
                    ),
                },
            ),
        ))
        .into()
    }

    pub(super) fn require_preparation_health(
        &self,
    ) -> Result<(), PhysicalMutationPreparationOutcome> {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            stale_preparation(PhysicalMutationPreparationStale::PublicationAuthorityReleased)
        })?;
        runtime
            .health
            .permit()
            .map(|_| ())
            .map_err(|()| map_record_denial(RecordAppendDenial::ServingRequiresInspection))
    }

    pub(super) fn group_queue_admission_tick(
        &self,
    ) -> Result<PhysicalGroupQueueAdmissionTick, PhysicalMutationPreparationOutcome> {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            stale_preparation(PhysicalMutationPreparationStale::PublicationAuthorityReleased)
        })?;
        let clock = runtime.signal.clock_observation().map_err(|_| {
            stale_preparation(PhysicalMutationPreparationStale::SignalOwnerUnavailable)
        })?;
        Ok(PhysicalGroupQueueAdmissionTick::new(clock.current_tick()))
    }

    fn preflight_durable_append(
        &self,
        batch: &RecordAppendBatch,
        placement: AdmittedRecordPlacementPolicy,
        manifest_capacity_transition: crate::physical_runtime::PhysicalManifestCapacityTransition,
    ) -> Result<(), PhysicalMutationPreparationOutcome> {
        if !placement.admits(self.format) {
            return Err(TransitionOutcome::denied(
                PhysicalMutationPreparationDenial::RecordAppend(
                    RecordAppendDenial::PlacementFormatMismatch,
                ),
            )
            .into());
        }
        batch.preflight(self.access).map_err(map_record_denial)?;
        preflight_placement(self.format, placement, batch).map_err(map_record_preflight)?;
        if manifest_capacity_transition
            == crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent
            && placement.manifest_capacity().get() != self.root_owner.snapshot().0.node_capacity()
        {
            return Err(map_record_denial(
                RecordAppendDenial::ManifestCapacityMigrationRequired,
            ));
        }
        Ok(())
    }

    pub(super) fn admit_mutation_preparation(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        manifest_capacity_transition: crate::physical_runtime::PhysicalManifestCapacityTransition,
        payload_digest: [u8; 32],
        request: PhysicalMutationRequest,
        family: PhysicalMutationOperationFamily,
    ) -> Result<PhysicalMutationPreparationAdmission, PhysicalMutationPreparationOutcome> {
        let (key, deadline, durability_request) = request.into_parts();
        let lease = key.lease();
        let fingerprint = self
            .derive_record_append_fingerprint(
                placement,
                manifest_capacity_transition,
                payload_digest,
                durability_request,
                family,
            )
            .map_err(|()| canonical_request_failure())?;
        let admission = self
            .admit_idempotency_binding(key, fingerprint)
            .map_err(map_idempotency_admission)?;
        let admission = match admission {
            PhysicalMutationIdempotencyRegistryAdmission::Fresh(binding) => {
                AdmittedPhysicalMutation::Fresh(binding)
            }
            PhysicalMutationIdempotencyRegistryAdmission::DuplicateUnresolved(existing) => {
                AdmittedPhysicalMutation::DuplicateUnresolved { existing, lease }
            }
            PhysicalMutationIdempotencyRegistryAdmission::ProvenNoEffect(terminal) => {
                return Ok(PhysicalMutationPreparationAdmission::ProvenNoEffect(
                    terminal,
                ))
            }
            PhysicalMutationIdempotencyRegistryAdmission::Completed(fact) => {
                return Ok(PhysicalMutationPreparationAdmission::Completed(
                    crate::physical_runtime::CompletedPhysicalMutation::from_fact(&fact),
                ))
            }
            PhysicalMutationIdempotencyRegistryAdmission::Indeterminate(fate) => {
                return Ok(PhysicalMutationPreparationAdmission::Indeterminate(fate))
            }
        };
        Ok(PhysicalMutationPreparationAdmission::Prepared(
            AdmittedMutationPreparation {
                admission,
                deadline,
            },
        ))
    }

    fn admit_idempotency_binding(
        &self,
        key: PhysicalMutationIdempotencyKey,
        fingerprint: PhysicalMutationRequestFingerprint,
    ) -> Result<
        PhysicalMutationIdempotencyRegistryAdmission,
        PhysicalMutationIdempotencyRegistryAdmissionError<PhysicalMutationIdentityReservationError>,
    > {
        self.idempotency
            .admit_unallocated_with(key, fingerprint, || {
                self.mutation_identity
                    .reserve_mutation_identity()
                    .map(|receipt| {
                        crate::physical_runtime::PhysicalMutationIdentity::from_reserved_operation(
                            receipt.identity(),
                        )
                    })
            })
    }
}

pub(super) fn stale_preparation(
    stale: PhysicalMutationPreparationStale,
) -> PhysicalMutationPreparationOutcome {
    TransitionOutcome::stale(stale).into()
}

fn canonical_payload(
    batch: RecordAppendBatch,
) -> Result<CanonicalRecordAppendPayload, PhysicalMutationPreparationOutcome> {
    prepare_canonical_payload(batch).map_err(|error| match error {
        CanonicalPayloadPreparationError::RecordSlots { required_records } => {
            TransitionOutcome::deferred(PhysicalMutationPreparationDeferred::PreparedRecordSlots {
                required_records,
            })
            .into()
        }
        CanonicalPayloadPreparationError::PayloadBytes { required_bytes } => {
            TransitionOutcome::deferred(PhysicalMutationPreparationDeferred::PreparedPayloadBytes {
                required_bytes,
            })
            .into()
        }
        CanonicalPayloadPreparationError::Failed(failure) => {
            TransitionOutcome::failed(PhysicalMutationPreparationFailure::Stream(failure)).into()
        }
    })
}

fn map_record_preflight(error: RecordAppendError) -> PhysicalMutationPreparationOutcome {
    match error {
        RecordAppendError::Denied(denial) => map_record_denial(denial),
        _ => canonical_request_failure(),
    }
}

pub(super) fn map_record_denial(denial: RecordAppendDenial) -> PhysicalMutationPreparationOutcome {
    TransitionOutcome::denied(PhysicalMutationPreparationDenial::RecordAppend(denial)).into()
}

pub(super) fn canonical_request_failure() -> PhysicalMutationPreparationOutcome {
    TransitionOutcome::failed(PhysicalMutationPreparationFailure::CanonicalRequestRejected).into()
}

fn map_idempotency_admission(
    error: PhysicalMutationIdempotencyRegistryAdmissionError<
        PhysicalMutationIdentityReservationError,
    >,
) -> PhysicalMutationPreparationOutcome {
    match error {
        PhysicalMutationIdempotencyRegistryAdmissionError::Reservation(error) => {
            map_identity_reservation(error)
        }
        PhysicalMutationIdempotencyRegistryAdmissionError::Denied(denial) => {
            map_idempotency_denial(denial)
        }
    }
}

fn map_idempotency_denial(
    denial: PhysicalMutationIdempotencyRegistryDenial,
) -> PhysicalMutationPreparationOutcome {
    match denial {
        PhysicalMutationIdempotencyRegistryDenial::AuthorityReleased => {
            TransitionOutcome::stale(PhysicalMutationPreparationStale::DurabilityAuthorityReleased)
                .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::ForeignStore
        | PhysicalMutationIdempotencyRegistryDenial::ForeignMutationStore => {
            TransitionOutcome::rebind_required(
                PhysicalMutationPreparationRebindRequired::ForeignStore,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::ForeignPolicy => {
            TransitionOutcome::rebind_required(
                PhysicalMutationPreparationRebindRequired::ForeignDurabilityPolicy,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::ForeignMutationRuntime => {
            TransitionOutcome::rebind_required(
                PhysicalMutationPreparationRebindRequired::ForeignRuntime,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::Expired => {
            TransitionOutcome::denied(PhysicalMutationPreparationDenial::IdempotencyExpired).into()
        }
        PhysicalMutationIdempotencyRegistryDenial::Conflict => {
            TransitionOutcome::denied(PhysicalMutationPreparationDenial::IdempotencyConflict).into()
        }
        PhysicalMutationIdempotencyRegistryDenial::PendingUnresolvedLimitReached => {
            TransitionOutcome::deferred(
                PhysicalMutationPreparationDeferred::PendingUnresolvedLimitReached,
            )
            .into()
        }
        PhysicalMutationIdempotencyRegistryDenial::LiveBindingLimitReached => {
            TransitionOutcome::deferred(
                PhysicalMutationPreparationDeferred::LiveBindingLimitReached,
            )
            .into()
        }
    }
}

fn map_identity_reservation(
    error: PhysicalMutationIdentityReservationError,
) -> PhysicalMutationPreparationOutcome {
    match error {
        PhysicalMutationIdentityReservationError::Stale(stale) => {
            let stale = match stale {
                PhysicalWorkSubmissionStale::OwnerReleased => {
                    PhysicalMutationPreparationStale::WorkOwnerReleased
                }
                PhysicalWorkSubmissionStale::LifecycleGenerationAdvanced => {
                    PhysicalMutationPreparationStale::LifecycleGenerationAdvanced
                }
                PhysicalWorkSubmissionStale::AdmissionStopped => {
                    PhysicalMutationPreparationStale::AdmissionStopped
                }
                PhysicalWorkSubmissionStale::SignalOwnerUnavailable => {
                    PhysicalMutationPreparationStale::SignalOwnerUnavailable
                }
            };
            TransitionOutcome::stale(stale).into()
        }
        PhysicalMutationIdentityReservationError::Failed(
            PhysicalWorkSubmissionFailure::OperationIdentityExhausted,
        ) => TransitionOutcome::failed(
            PhysicalMutationPreparationFailure::OperationIdentityExhausted,
        )
        .into(),
    }
}

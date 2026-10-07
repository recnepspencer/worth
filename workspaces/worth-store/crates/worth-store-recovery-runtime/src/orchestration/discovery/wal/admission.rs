use worth_store::physical_runtime::{
    recovery_wal::{wal_frame_integrity_scope_identity, WalSegmentArtifactIdentity},
    IntegrityAdmittedRecoveryWalFrame, IntegrityAdmittedRecoveryWalSegmentBuilder,
    ObservedWalArtifact, PhysicalRecoveryCoordination, RecoveryWalAllocationDenial,
    RecoveryWalIntegrityAdmissionDenial,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::{
    validate_wal_frame_prefix, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use crate::entry::{
    PhysicalRecoveryWalIntegrityObservation, PhysicalRecoveryWalInventoryAllocationBoundary,
    WalIntegrityObservationBuilder,
};
use crate::integrity_ingress::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressCounters,
    RecoveryIntegrityIngressRejection,
};

use super::observation_projection::public_observation;

/// Admission truth for one exact C4 source; C8 classifies its final disposition.
pub(super) struct WalSegmentAdmissionTranscript<'owner, 'source> {
    pub identity: WalSegmentArtifactIdentity,
    pub name: &'source std::ffi::OsStr,
    pub observed_bytes: u64,
    pub frames: IntegrityAdmittedRecoveryWalSegmentBuilder<'owner, 'source>,
    pub rejection: Option<PhysicalIntegrityRejection>,
    pub counters: RecoveryIntegrityIngressCounters,
}

pub(super) enum WalSegmentAdmissionDenial {
    CounterOverflow,
    FrameLimitExceeded {
        observed: u64,
        admitted: u64,
    },
    SourceBinding,
    Allocation(RecoveryWalAllocationDenial),
    InventoryAllocation {
        boundary: PhysicalRecoveryWalInventoryAllocationBoundary,
        cause: RecoveryWalAllocationDenial,
    },
}

pub(super) struct WalSegmentAdmissionFailure {
    pub denial: WalSegmentAdmissionDenial,
    pub counters: RecoveryIntegrityIngressCounters,
    pub policy_attempts: u64,
}

enum FrameAdmission {
    Admitted {
        frame: IntegrityAdmittedRecoveryWalFrame,
        observation: PhysicalRecoveryWalIntegrityObservation,
        next_offset: usize,
    },
    Rejected {
        rejection: PhysicalIntegrityRejection,
        observation: PhysicalRecoveryWalIntegrityObservation,
    },
}

pub(super) fn admit_segment<'owner, 'source>(
    owner: &'owner PhysicalRecoveryCoordination,
    identity: WalSegmentArtifactIdentity,
    artifact: &'source ObservedWalArtifact,
    store: StableStoreIdentity,
    maximum_attempts: u64,
    observations: &mut WalIntegrityObservationBuilder,
) -> Result<WalSegmentAdmissionTranscript<'owner, 'source>, WalSegmentAdmissionFailure> {
    let frames = owner
        .begin_recovery_wal_segment(artifact, identity)
        .map_err(|denial| WalSegmentAdmissionFailure {
            denial: owner_denial(denial),
            counters: RecoveryIntegrityIngressCounters::default(),
            policy_attempts: 0,
        })?;
    let mut transcript = WalSegmentAdmissionTranscript {
        identity,
        name: artifact.name(),
        observed_bytes: artifact.bytes().map_or(0, |bytes| bytes.len() as u64),
        frames,
        rejection: None,
        counters: RecoveryIntegrityIngressCounters::default(),
    };
    let mut offset = 0_usize;
    let bytes_len = artifact.bytes().map_or(0, <[u8]>::len);
    let mut empty_source_pending = bytes_len == 0;
    while offset < bytes_len || empty_source_pending {
        empty_source_pending = false;
        if bytes_len != 0 {
            let Some(observed) = (transcript.frames.len() as u64).checked_add(1) else {
                return Err(failure(
                    WalSegmentAdmissionDenial::CounterOverflow,
                    transcript,
                ));
            };
            if observed > maximum_attempts {
                return Err(failure(
                    WalSegmentAdmissionDenial::FrameLimitExceeded {
                        observed,
                        admitted: maximum_attempts,
                    },
                    transcript,
                ));
            }
        }
        if let Err(cause) = observations.reserve_one(owner) {
            return Err(failure(
                WalSegmentAdmissionDenial::InventoryAllocation {
                    boundary: PhysicalRecoveryWalInventoryAllocationBoundary::IntegrityObservations,
                    cause,
                },
                transcript,
            ));
        }
        match admit_frame(
            owner,
            artifact,
            identity,
            store,
            offset,
            &mut transcript.counters,
        ) {
            Ok(FrameAdmission::Admitted {
                frame,
                observation,
                next_offset,
            }) => {
                observations.push_reserved(observation);
                offset = next_offset;
                if let Err(denial) = transcript.frames.push(frame) {
                    return Err(failure(owner_denial(denial), transcript));
                }
            }
            Ok(FrameAdmission::Rejected {
                rejection,
                observation,
            }) => {
                observations.push_reserved(observation);
                transcript.rejection = Some(rejection);
                break;
            }
            Err(denial) => return Err(failure(denial, transcript)),
        }
    }
    Ok(transcript)
}

fn admit_frame(
    owner: &PhysicalRecoveryCoordination,
    artifact: &ObservedWalArtifact,
    identity: WalSegmentArtifactIdentity,
    store: StableStoreIdentity,
    offset: usize,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> Result<FrameAdmission, WalSegmentAdmissionDenial> {
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(
        &artifact.bytes().unwrap_or_default()[offset..],
    );
    let (validation, _) = validate_wal_frame_prefix(
        input,
        store,
        wal_frame_integrity_scope_identity(identity),
        offset as u64,
    );
    let scope = match &validation {
        worth_store_physical_integrity::WalFrameIntegrityValidation::Intact(value) => value.scope(),
        worth_store_physical_integrity::WalFrameIntegrityValidation::Rejected(value) => {
            value.scope()
        }
    };
    let attempt = IntegrityAdmittedRecoveryArtifact::bind_wal_frame(
        owner,
        artifact,
        scope,
        scope.byte_range(),
        validation,
        counters,
    )
    .map_err(WalSegmentAdmissionDenial::Allocation)?;
    let observation = attempt.observation();
    match attempt.into_outcome() {
        Ok(IntegrityAdmittedRecoveryArtifact::WalFrame(frame)) => {
            let frame = frame.into_owner_redo_projection(counters);
            let next_offset = usize::try_from(scope.byte_range().end_exclusive())
                .map_err(|_| WalSegmentAdmissionDenial::CounterOverflow)?;
            Ok(FrameAdmission::Admitted {
                frame,
                observation: public_observation(observation),
                next_offset,
            })
        }
        Ok(_) => unreachable!("WAL ingress routes only WAL frames"),
        Err(RecoveryIntegrityIngressRejection::Integrity(rejection)) => {
            Ok(FrameAdmission::Rejected {
                rejection,
                observation: public_observation(observation),
            })
        }
        Err(_) => Err(WalSegmentAdmissionDenial::SourceBinding),
    }
}

pub(super) fn owner_denial(
    denial: RecoveryWalIntegrityAdmissionDenial,
) -> WalSegmentAdmissionDenial {
    match denial {
        RecoveryWalIntegrityAdmissionDenial::Allocation(cause) => {
            WalSegmentAdmissionDenial::Allocation(cause)
        }
        _ => WalSegmentAdmissionDenial::SourceBinding,
    }
}

fn failure(
    denial: WalSegmentAdmissionDenial,
    transcript: WalSegmentAdmissionTranscript<'_, '_>,
) -> WalSegmentAdmissionFailure {
    let policy_attempts = super::inventory_accumulation::policy_attempts(&transcript);
    WalSegmentAdmissionFailure {
        denial,
        counters: transcript.counters,
        policy_attempts,
    }
}

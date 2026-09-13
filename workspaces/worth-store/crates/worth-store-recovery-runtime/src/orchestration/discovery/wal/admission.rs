use worth_store::physical_runtime::{
    recovery_wal::{wal_frame_integrity_scope_identity, WalSegmentArtifactIdentity},
    IntegrityAdmittedRecoveryWalFrame, ObservedWalArtifact,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::{
    validate_wal_frame_prefix, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use crate::entry::PhysicalRecoveryWalIntegrityObservation;
use crate::integrity_ingress::{
    IntegrityAdmittedRecoveryArtifact, RecoveryIntegrityIngressCounters,
    RecoveryIntegrityIngressRejection,
};

use super::observation_projection::public_observation;

/// C.9 transcript for one exact C.4-observed WAL artifact.
///
/// This records admission truth only. C.8 decides whether a rejection is a
/// lawful torn tail, corruption, or residue after all canonical sources are
/// known.
pub(super) struct WalSegmentAdmissionTranscript {
    pub identity: WalSegmentArtifactIdentity,
    pub name: String,
    pub artifact: ObservedWalArtifact,
    pub observed_bytes: u64,
    pub frames: Vec<IntegrityAdmittedRecoveryWalFrame>,
    pub observations: Vec<PhysicalRecoveryWalIntegrityObservation>,
    pub rejection: Option<PhysicalIntegrityRejection>,
    pub counters: RecoveryIntegrityIngressCounters,
}

#[derive(Clone, Copy)]
pub(super) enum WalSegmentAdmissionDenial {
    CounterOverflow,
    FrameLimitExceeded { observed: u64, admitted: u64 },
    SourceBinding,
}

pub(super) struct WalSegmentAdmissionFailure {
    pub denial: WalSegmentAdmissionDenial,
    pub transcript: WalSegmentAdmissionTranscript,
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

pub(super) fn admit_segment(
    owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
    identity: WalSegmentArtifactIdentity,
    artifact: ObservedWalArtifact,
    store: StableStoreIdentity,
    maximum_attempts: u64,
) -> Result<WalSegmentAdmissionTranscript, WalSegmentAdmissionFailure> {
    let name = artifact.name().to_string_lossy().into_owned();
    let observed_bytes = artifact.bytes().map_or(0, |bytes| bytes.len() as u64);
    let mut offset = 0_usize;
    let mut transcript = WalSegmentAdmissionTranscript {
        identity,
        name,
        artifact,
        observed_bytes,
        frames: Vec::new(),
        observations: Vec::new(),
        rejection: None,
        counters: RecoveryIntegrityIngressCounters::default(),
    };
    let bytes_len = transcript.artifact.bytes().map_or(0, <[u8]>::len);
    while offset < bytes_len || (bytes_len == 0 && transcript.observations.is_empty()) {
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
        match admit_frame(
            owner,
            &transcript.artifact,
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
                transcript.observations.push(observation);
                offset = next_offset;
                transcript.frames.push(frame);
            }
            Ok(FrameAdmission::Rejected {
                rejection,
                observation,
            }) => {
                transcript.observations.push(observation);
                transcript.rejection = Some(rejection);
                break;
            }
            Err(denial) => return Err(failure(denial, transcript)),
        }
    }
    Ok(transcript)
}

fn admit_frame(
    owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
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
    );
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

fn failure(
    denial: WalSegmentAdmissionDenial,
    transcript: WalSegmentAdmissionTranscript,
) -> WalSegmentAdmissionFailure {
    WalSegmentAdmissionFailure { denial, transcript }
}

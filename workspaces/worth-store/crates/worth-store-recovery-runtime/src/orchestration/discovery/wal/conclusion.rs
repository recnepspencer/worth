use worth_store::physical_runtime::IntegrityAdmittedRecoveryWalSegment;
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_physics::{
    classify_admitted_wal_segment, AdmittedWalFrameRejectionKind, AdmittedWalSegmentPolicyInput,
    PhysicalRecoveryResidue, PhysicalWalFrameFacts, PhysicalWalSegmentCandidate,
    PhysicalWalSegmentDisposition,
};

use crate::entry::{PhysicalRecoveryWalIntegrityDenial, PhysicalRecoveryWalIntegrityObservation};

use super::admission::WalSegmentAdmissionTranscript;

pub(super) struct WalSegmentConclusion {
    pub candidate: Option<PhysicalWalSegmentCandidate>,
    pub admitted: Option<IntegrityAdmittedRecoveryWalSegment>,
    pub residue: Vec<PhysicalRecoveryResidue>,
    pub corruptions: Vec<PhysicalRecoveryWalIntegrityDenial>,
    pub observations: Vec<PhysicalRecoveryWalIntegrityObservation>,
    pub valid_frames: u64,
    pub valid_bytes: u64,
    pub torn_bytes: u64,
}

pub(super) struct WalSegmentConclusionFailure {
    pub observations: Vec<PhysicalRecoveryWalIntegrityObservation>,
}

pub(super) fn conclude_segment(
    owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
    transcript: WalSegmentAdmissionTranscript,
    terminal: bool,
) -> Result<WalSegmentConclusion, WalSegmentConclusionFailure> {
    let admitted = if transcript.frames.is_empty() {
        None
    } else {
        let Some(admitted) = owner.retain_admitted_recovery_wal_segment(
            &transcript.artifact,
            transcript.identity,
            transcript.frames,
        ) else {
            return Err(WalSegmentConclusionFailure {
                observations: transcript.observations,
            });
        };
        Some(admitted)
    };
    let prefix = admitted.as_ref().map(|segment| {
        let facts = segment
            .frames()
            .iter()
            .map(|frame| {
                PhysicalWalFrameFacts::new(frame.lsn_range(), frame.encoded_byte_count())
                    .expect("C.9 admitted WAL frame has nonzero bytes")
            })
            .collect();
        (segment.inspection(), facts)
    });
    let rejection_kind = transcript.rejection.map(|rejection| {
        if is_truncation(rejection) {
            AdmittedWalFrameRejectionKind::Truncated
        } else {
            AdmittedWalFrameRejectionKind::Other
        }
    });
    let disposition = classify_admitted_wal_segment(AdmittedWalSegmentPolicyInput::new(
        transcript.name.clone(),
        transcript.identity,
        transcript.observed_bytes,
        terminal,
        rejection_kind,
        prefix,
    ))
    .ok_or_else(|| WalSegmentConclusionFailure {
        observations: transcript.observations.clone(),
    })?;
    let mut conclusion = WalSegmentConclusion {
        candidate: None,
        admitted: None,
        residue: Vec::new(),
        corruptions: Vec::new(),
        observations: transcript.observations,
        valid_frames: 0,
        valid_bytes: 0,
        torn_bytes: 0,
    };
    match disposition {
        PhysicalWalSegmentDisposition::Candidate {
            candidate,
            torn_bytes,
        } => {
            let inspection = candidate.inspection();
            conclusion.valid_frames = inspection.frame_count();
            conclusion.valid_bytes = inspection.byte_count();
            conclusion.torn_bytes = torn_bytes;
            conclusion.candidate = Some(candidate);
            conclusion.admitted = admitted;
        }
        PhysicalWalSegmentDisposition::Residue {
            residue,
            torn_bytes,
        } => {
            conclusion.torn_bytes = torn_bytes;
            conclusion.residue.push(residue);
        }
        PhysicalWalSegmentDisposition::Corrupt => {
            if let Some(rejection) = transcript.rejection {
                conclusion
                    .corruptions
                    .push(PhysicalRecoveryWalIntegrityDenial::new(
                        transcript.name,
                        transcript.identity,
                        rejection,
                    ));
            }
        }
    }
    Ok(conclusion)
}

fn is_truncation(rejection: PhysicalIntegrityRejection) -> bool {
    matches!(
        rejection,
        PhysicalIntegrityRejection::Damaged(localization)
            if localization.cause() == PhysicalDamageCause::Truncated
    )
}

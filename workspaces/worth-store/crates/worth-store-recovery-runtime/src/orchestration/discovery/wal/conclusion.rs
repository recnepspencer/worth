use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalSegment, PhysicalRecoveryCoordination,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_physics::{AdmittedWalFrameRejectionKind, PhysicalRecoveryResidue};

use crate::entry::{
    PhysicalRecoveryWalIntegrityDenial, PhysicalRecoveryWalInventoryAllocationBoundary,
};
use crate::orchestration::wal_selection::{
    CandidateClassificationDenial, RecordedWalDisposition, ResidentWalCandidates,
};

use super::admission::WalSegmentAdmissionTranscript;

pub(super) struct WalSegmentConclusion {
    pub admitted: Option<IntegrityAdmittedRecoveryWalSegment>,
    pub residue: Vec<PhysicalRecoveryResidue>,
    pub corruptions: Vec<PhysicalRecoveryWalIntegrityDenial>,
    pub valid_frames: u64,
    pub valid_bytes: u64,
    pub torn_bytes: u64,
}

pub(super) struct WalSegmentConclusionFailure {
    pub kind: super::WalDiscoveryInventoryDenialKind,
}

pub(super) fn conclude_segment(
    transcript: WalSegmentAdmissionTranscript<'_, '_>,
    terminal: bool,
    owner: &PhysicalRecoveryCoordination,
    candidates: &mut ResidentWalCandidates,
) -> Result<WalSegmentConclusion, WalSegmentConclusionFailure> {
    let admitted = if transcript.frames.is_empty() {
        None
    } else {
        let admitted = match transcript.frames.finish() {
            Ok(admitted) => admitted,
            Err(denial) => {
                return Err(WalSegmentConclusionFailure {
                kind: match denial {
                    worth_store::physical_runtime::RecoveryWalIntegrityAdmissionDenial::Allocation(cause) => super::WalDiscoveryInventoryDenialKind::Allocation(cause),
                    _ => super::WalDiscoveryInventoryDenialKind::SourceBinding,
                },
            });
            }
        };
        Some(admitted)
    };
    let rejection_kind = transcript.rejection.map(|rejection| {
        if is_truncation(rejection) {
            AdmittedWalFrameRejectionKind::Truncated
        } else {
            AdmittedWalFrameRejectionKind::Other
        }
    });
    let disposition = candidates
        .classify(
            owner,
            transcript.identity,
            transcript.observed_bytes,
            terminal,
            rejection_kind,
            admitted.as_ref(),
        )
        .map_err(|denial| WalSegmentConclusionFailure {
            kind: match denial {
                CandidateClassificationDenial::Allocation(cause) => {
                    super::WalDiscoveryInventoryDenialKind::InventoryAllocation {
                        boundary:
                            PhysicalRecoveryWalInventoryAllocationBoundary::CandidateFrameFacts,
                        cause,
                    }
                }
                CandidateClassificationDenial::SourceBinding => {
                    super::WalDiscoveryInventoryDenialKind::SourceBinding
                }
            },
        })?;
    let mut conclusion = WalSegmentConclusion {
        admitted: None,
        residue: Vec::new(),
        corruptions: Vec::new(),
        valid_frames: 0,
        valid_bytes: 0,
        torn_bytes: 0,
    };
    match disposition {
        RecordedWalDisposition::Candidate {
            inspection,
            torn_bytes,
        } => {
            conclusion.valid_frames = inspection.frame_count();
            conclusion.valid_bytes = inspection.byte_count();
            conclusion.torn_bytes = torn_bytes;
            conclusion.admitted = admitted;
        }
        RecordedWalDisposition::Residue {
            kind,
            observed_bytes,
            torn_bytes,
        } => {
            conclusion.torn_bytes = torn_bytes;
            conclusion
                .residue
                .push(PhysicalRecoveryResidue::with_observed_bytes(
                    transcript.name.to_string_lossy().into_owned(),
                    kind,
                    observed_bytes,
                ));
        }
        RecordedWalDisposition::Corrupt => {
            if let Some(rejection) = transcript.rejection {
                conclusion
                    .corruptions
                    .push(PhysicalRecoveryWalIntegrityDenial::new(
                        transcript.name.to_string_lossy().into_owned(),
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

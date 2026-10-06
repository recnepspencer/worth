use worth_store_recovery_physics::PhysicalRecoveryResidue;

use super::admission::{
    WalSegmentAdmissionDenial, WalSegmentAdmissionFailure, WalSegmentAdmissionTranscript,
};
use super::conclusion::{WalSegmentConclusion, WalSegmentConclusionFailure};
use super::{
    AdmittedWalInventory, RecoveryIntegrityIngressCounters, WalDiscoveryInventory,
    WalDiscoveryInventoryDenial, WalDiscoveryInventoryDenialKind,
};

impl WalDiscoveryInventory {
    pub(super) fn new(
        canonical_segments: u64,
        observed_bytes: u64,
        residue: Vec<PhysicalRecoveryResidue>,
    ) -> Self {
        Self {
            candidates: crate::orchestration::wal_selection::ResidentWalCandidates::default(),
            admitted: AdmittedWalInventory::default(),
            residue,
            corruptions: Vec::new(),
            observations: crate::entry::WalIntegrityObservationBuilder::new(),
            ingress: RecoveryIntegrityIngressCounters::default(),
            canonical_segments,
            frames_scanned: 0,
            valid_frames: 0,
            valid_bytes: 0,
            observed_bytes,
            torn_suffix_frames: 0,
            torn_suffix_bytes: 0,
        }
    }

    pub(super) fn record_ingress(&mut self, counters: RecoveryIntegrityIngressCounters) -> bool {
        let Some(ingress) = self.ingress.checked_add(counters) else {
            return false;
        };
        self.ingress = ingress;
        true
    }

    pub(super) fn record_conclusion(
        &mut self,
        attempted: u64,
        conclusion: WalSegmentConclusion,
    ) -> bool {
        let Some(valid_frames) = self.valid_frames.checked_add(conclusion.valid_frames) else {
            return false;
        };
        let Some(valid_bytes) = self.valid_bytes.checked_add(conclusion.valid_bytes) else {
            return false;
        };
        let Some(torn_frames) = self
            .torn_suffix_frames
            .checked_add(u64::from(conclusion.torn_bytes != 0))
        else {
            return false;
        };
        let Some(torn_bytes) = self.torn_suffix_bytes.checked_add(conclusion.torn_bytes) else {
            return false;
        };
        self.frames_scanned = attempted;
        self.valid_frames = valid_frames;
        self.valid_bytes = valid_bytes;
        self.torn_suffix_frames = torn_frames;
        self.torn_suffix_bytes = torn_bytes;
        self.residue.extend(conclusion.residue);
        self.corruptions.extend(conclusion.corruptions);
        if let Some(admitted) = conclusion.admitted {
            self.admitted.push(admitted);
        }
        true
    }

    pub(super) fn deny_admission(
        mut self,
        failure: WalSegmentAdmissionFailure,
    ) -> WalDiscoveryInventoryDenial {
        self.frames_scanned = self.frames_scanned.saturating_add(failure.policy_attempts);
        if !self.record_ingress(failure.counters) {
            return self.deny(WalDiscoveryInventoryDenialKind::CounterOverflow);
        }
        let kind = match failure.denial {
            WalSegmentAdmissionDenial::CounterOverflow => {
                WalDiscoveryInventoryDenialKind::CounterOverflow
            }
            WalSegmentAdmissionDenial::FrameLimitExceeded { observed, admitted } => {
                WalDiscoveryInventoryDenialKind::FrameLimitExceeded { observed, admitted }
            }
            WalSegmentAdmissionDenial::SourceBinding => {
                WalDiscoveryInventoryDenialKind::SourceBinding
            }
            WalSegmentAdmissionDenial::Allocation(cause) => {
                WalDiscoveryInventoryDenialKind::Allocation(cause)
            }
            WalSegmentAdmissionDenial::InventoryAllocation { boundary, cause } => {
                WalDiscoveryInventoryDenialKind::InventoryAllocation { boundary, cause }
            }
        };
        self.deny(kind)
    }

    pub(super) fn deny_conclusion(
        self,
        failure: WalSegmentConclusionFailure,
    ) -> WalDiscoveryInventoryDenial {
        self.deny(failure.kind)
    }

    pub(super) fn deny(self, kind: WalDiscoveryInventoryDenialKind) -> WalDiscoveryInventoryDenial {
        WalDiscoveryInventoryDenial {
            kind,
            inventory: self,
        }
    }
}

pub(super) fn policy_attempts(transcript: &WalSegmentAdmissionTranscript<'_, '_>) -> u64 {
    if transcript.observed_bytes == 0 {
        0
    } else {
        transcript.counters.attempted
    }
}

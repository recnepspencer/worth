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
            candidates: Vec::new(),
            admitted: AdmittedWalInventory::default(),
            residue,
            corruptions: Vec::new(),
            observations: Vec::new(),
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
        self.observations.extend(conclusion.observations);
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
        if let Some(candidate) = conclusion.candidate {
            self.candidates.push(candidate);
        }
        if let Some(admitted) = conclusion.admitted {
            self.admitted.push(admitted);
        }
        true
    }

    pub(super) fn deny_admission(
        mut self,
        failure: WalSegmentAdmissionFailure,
    ) -> WalDiscoveryInventoryDenial {
        let prior_frames = self.frames_scanned;
        self.frames_scanned = prior_frames.saturating_add(policy_attempts(&failure.transcript));
        self.observations.extend(failure.transcript.observations);
        if !self.record_ingress(failure.transcript.counters) {
            return self.deny(WalDiscoveryInventoryDenialKind::CounterOverflow);
        }
        let kind = match failure.denial {
            WalSegmentAdmissionDenial::CounterOverflow => {
                WalDiscoveryInventoryDenialKind::CounterOverflow
            }
            WalSegmentAdmissionDenial::FrameLimitExceeded { observed, admitted } => {
                WalDiscoveryInventoryDenialKind::FrameLimitExceeded {
                    observed: prior_frames.saturating_add(observed),
                    admitted: prior_frames.saturating_add(admitted),
                }
            }
            WalSegmentAdmissionDenial::SourceBinding => {
                WalDiscoveryInventoryDenialKind::SourceBinding
            }
        };
        self.deny(kind)
    }

    pub(super) fn deny_conclusion(
        mut self,
        failure: WalSegmentConclusionFailure,
    ) -> WalDiscoveryInventoryDenial {
        self.observations.extend(failure.observations);
        self.deny(WalDiscoveryInventoryDenialKind::SourceBinding)
    }

    pub(super) fn deny(self, kind: WalDiscoveryInventoryDenialKind) -> WalDiscoveryInventoryDenial {
        WalDiscoveryInventoryDenial {
            kind,
            inventory: self,
        }
    }
}

pub(super) fn policy_attempts(transcript: &WalSegmentAdmissionTranscript) -> u64 {
    if transcript.observed_bytes == 0 {
        0
    } else {
        transcript.counters.attempted
    }
}

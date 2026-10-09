use worth_store_wal::{WalSegmentArtifactIdentity, WalSegmentInspection};

use super::{
    PhysicalRecoveryResidueKind, PhysicalWalFrameFacts, PhysicalWalInterruptionFacts,
    PhysicalWalSegmentCandidate,
};

/// C.8's recovery-policy view of one C.9 admission transcript.
pub struct AdmittedWalSegmentPolicyInput {
    identity: WalSegmentArtifactIdentity,
    observed_bytes: u64,
    terminal: bool,
    rejection: Option<AdmittedWalFrameRejectionKind>,
    prefix: Option<WalSegmentInspection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmittedWalFrameRejectionKind {
    Truncated,
    Other,
}

pub enum PhysicalWalSegmentDisposition {
    Candidate {
        preparation: PhysicalWalCandidatePreparation,
        torn_bytes: u64,
    },
    Residue {
        kind: PhysicalRecoveryResidueKind,
        observed_bytes: u64,
        torn_bytes: u64,
    },
    Corrupt,
}

/// Policy eligibility for an admitted prefix, not yet bound to its frame facts.
/// Only the classifier creates this preparation; binding preserves the supplied
/// storage and the candidate's existing exact-prefix checks.
#[derive(Debug)]
pub struct PhysicalWalCandidatePreparation {
    inspection: WalSegmentInspection,
    interruption: Option<PhysicalWalInterruptionFacts>,
}

impl PhysicalWalCandidatePreparation {
    pub fn bind_frame_facts(
        self,
        frame_facts: Vec<PhysicalWalFrameFacts>,
    ) -> Option<PhysicalWalSegmentCandidate> {
        PhysicalWalSegmentCandidate::from_frame_facts(
            self.inspection,
            self.interruption,
            frame_facts,
        )
    }
}

impl AdmittedWalSegmentPolicyInput {
    pub fn new(
        identity: WalSegmentArtifactIdentity,
        observed_bytes: u64,
        terminal: bool,
        rejection: Option<AdmittedWalFrameRejectionKind>,
        prefix: Option<WalSegmentInspection>,
    ) -> Self {
        Self {
            identity,
            observed_bytes,
            terminal,
            rejection,
            prefix,
        }
    }
}

pub fn classify_admitted_wal_segment(
    input: AdmittedWalSegmentPolicyInput,
) -> Option<PhysicalWalSegmentDisposition> {
    if input.observed_bytes == 0 && input.terminal {
        return Some(PhysicalWalSegmentDisposition::Residue {
            kind: PhysicalRecoveryResidueKind::TrailingEmptyWalSegment,
            observed_bytes: 0,
            torn_bytes: 0,
        });
    }
    let terminal_truncation =
        input.terminal && input.rejection == Some(AdmittedWalFrameRejectionKind::Truncated);
    let Some(inspection) = input.prefix else {
        return Some(if terminal_truncation {
            PhysicalWalSegmentDisposition::Residue {
                kind: PhysicalRecoveryResidueKind::InterruptedWalSegmentStart,
                observed_bytes: input.observed_bytes,
                torn_bytes: input.observed_bytes,
            }
        } else {
            PhysicalWalSegmentDisposition::Corrupt
        });
    };
    if input.rejection.is_some() && !terminal_truncation {
        return Some(PhysicalWalSegmentDisposition::Corrupt);
    }
    if inspection.identity() != input.identity {
        return None;
    }
    let interruption = terminal_truncation.then(|| {
        PhysicalWalInterruptionFacts::new(inspection.byte_count(), input.observed_bytes)
            .expect("terminal rejection follows a nonempty admitted prefix")
    });
    Some(PhysicalWalSegmentDisposition::Candidate {
        preparation: PhysicalWalCandidatePreparation {
            inspection,
            interruption,
        },
        torn_bytes: interruption
            .map_or(0, |tail| tail.observed_bytes() - tail.valid_prefix_bytes()),
    })
}

#[cfg(test)]
mod tests;

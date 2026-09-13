use worth_store_wal::{WalSegmentArtifactIdentity, WalSegmentInspection};

use super::{
    PhysicalRecoveryResidue, PhysicalRecoveryResidueKind, PhysicalWalFrameFacts,
    PhysicalWalInterruptionFacts, PhysicalWalSegmentCandidate,
};

/// C.8's recovery-policy view of one C.9 admission transcript.
pub struct AdmittedWalSegmentPolicyInput {
    name: String,
    identity: WalSegmentArtifactIdentity,
    observed_bytes: u64,
    terminal: bool,
    rejection: Option<AdmittedWalFrameRejectionKind>,
    prefix: Option<(WalSegmentInspection, Vec<PhysicalWalFrameFacts>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmittedWalFrameRejectionKind {
    Truncated,
    Other,
}

pub enum PhysicalWalSegmentDisposition {
    Candidate {
        candidate: PhysicalWalSegmentCandidate,
        torn_bytes: u64,
    },
    Residue {
        residue: PhysicalRecoveryResidue,
        torn_bytes: u64,
    },
    Corrupt,
}

impl AdmittedWalSegmentPolicyInput {
    pub fn new(
        name: String,
        identity: WalSegmentArtifactIdentity,
        observed_bytes: u64,
        terminal: bool,
        rejection: Option<AdmittedWalFrameRejectionKind>,
        prefix: Option<(WalSegmentInspection, Vec<PhysicalWalFrameFacts>)>,
    ) -> Self {
        Self {
            name,
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
            residue: PhysicalRecoveryResidue::new(
                input.name,
                PhysicalRecoveryResidueKind::TrailingEmptyWalSegment,
            ),
            torn_bytes: 0,
        });
    }
    let terminal_truncation =
        input.terminal && input.rejection == Some(AdmittedWalFrameRejectionKind::Truncated);
    let Some((inspection, frame_facts)) = input.prefix else {
        return Some(if terminal_truncation {
            PhysicalWalSegmentDisposition::Residue {
                residue: PhysicalRecoveryResidue::with_observed_bytes(
                    input.name,
                    PhysicalRecoveryResidueKind::InterruptedWalSegmentStart,
                    input.observed_bytes,
                ),
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
    let candidate =
        PhysicalWalSegmentCandidate::from_frame_facts(inspection, interruption, frame_facts)?;
    Some(PhysicalWalSegmentDisposition::Candidate {
        candidate,
        torn_bytes: interruption
            .map_or(0, |tail| tail.observed_bytes() - tail.valid_prefix_bytes()),
    })
}

#[cfg(test)]
mod tests;

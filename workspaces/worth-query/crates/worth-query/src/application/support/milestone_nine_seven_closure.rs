use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryConcurrentHostileMatrixArtifact, WorthQueryConcurrentHostileMatrixPosture,
    WorthQueryMilestoneClosureStatus, WorthQueryPublicBridgeReaderLaneCertification,
    WorthQueryPublicBridgeReaderLanePosture,
};
#[cfg(test)]
use super::{
    WorthQueryJournalIdentityBoundaryPosture, WorthQueryJournalReplayBoundaryCertification,
    WorthQuerySharedReadPinningBoundaryClosure, WorthQuerySharedReadPinningBoundaryPosture,
};

const REQUIRED_PHASES: [&str; 4] = [
    "phase-13-shared-read-pinning",
    "phase-15-journal-replay",
    "phase-16-concurrent-hostile-matrix",
    "phase-17-public-bridge-reader-lane",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMilestoneNineSevenPhaseClosure {
    phase: &'static str,
    status: WorthQueryMilestoneClosureStatus,
    evidence_digest: String,
}

impl WorthQueryMilestoneNineSevenPhaseClosure {
    #[cfg(test)]
    pub fn from_shared_read_pinning(closure: &WorthQuerySharedReadPinningBoundaryClosure) -> Self {
        Self::new(
            REQUIRED_PHASES[0],
            milestone_status_from_pinning_posture(closure.posture()),
            closure.closure_digest(),
        )
    }

    #[cfg(test)]
    pub fn from_journal_replay_boundary(
        closure: &WorthQueryJournalReplayBoundaryCertification,
    ) -> Self {
        Self::new(
            REQUIRED_PHASES[1],
            milestone_status_from_journal_posture(closure.journal_boundary_posture()),
            closure.journal_identity_inventory_digest(),
        )
    }

    pub fn from_concurrent_hostile_matrix(
        artifact: &WorthQueryConcurrentHostileMatrixArtifact,
    ) -> Self {
        Self::new(
            REQUIRED_PHASES[2],
            milestone_status_from_concurrent_posture(artifact.posture()),
            artifact.digest().as_str(),
        )
    }

    pub fn from_public_bridge_reader_lane(
        certification: &WorthQueryPublicBridgeReaderLaneCertification,
    ) -> Self {
        Self::new(
            REQUIRED_PHASES[3],
            milestone_status_from_public_bridge_posture(certification.posture()),
            certification.digest().as_str(),
        )
    }

    pub(crate) fn new(
        phase: &'static str,
        status: WorthQueryMilestoneClosureStatus,
        evidence_digest: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            status,
            evidence_digest: evidence_digest.into(),
        }
    }

    pub fn phase(&self) -> &'static str {
        self.phase
    }

    pub fn status(&self) -> WorthQueryMilestoneClosureStatus {
        self.status
    }

    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMilestoneNineSevenDerivedClosure {
    status: WorthQueryMilestoneClosureStatus,
    phase_closures: Vec<WorthQueryMilestoneNineSevenPhaseClosure>,
    defended_exclusions: Vec<String>,
    closure_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryMilestoneNineSevenDerivedClosure {
    pub fn derive_from_phase_closures(
        phase_closures: impl IntoIterator<Item = WorthQueryMilestoneNineSevenPhaseClosure>,
        defended_exclusions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut phase_closures = phase_closures.into_iter().collect::<Vec<_>>();
        phase_closures.sort_by_key(WorthQueryMilestoneNineSevenPhaseClosure::phase);
        let defended_exclusions = defended_exclusions
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let status = derive_milestone_nine_seven_status(&phase_closures);
        let closure_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_shape(WorthQueryEvidenceTag::new("milestone"), "worth-query-9.7")
                .field_shape(WorthQueryEvidenceTag::new("status"), status.as_str())
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("required_phase"),
                    REQUIRED_PHASES.iter().copied(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("phase_status"),
                    phase_closures.iter().map(|closure| {
                        format!("{}:{}", closure.phase(), closure.status().as_str())
                    }),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("phase_evidence_digest"),
                    phase_closures
                        .iter()
                        .map(WorthQueryMilestoneNineSevenPhaseClosure::evidence_digest),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("defended_exclusion"),
                    defended_exclusions.iter().map(String::as_str),
                )
                .seal()?;
        Ok(Self {
            status,
            phase_closures,
            defended_exclusions,
            closure_identity,
        })
    }

    pub fn support_profile_publication_contract() -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::derive_from_phase_closures(
            REQUIRED_PHASES.into_iter().map(|phase| {
                WorthQueryMilestoneNineSevenPhaseClosure::new(
                    phase,
                    WorthQueryMilestoneClosureStatus::Partial,
                    format!("{phase}:support-profile-requires-phase-local-artifact"),
                )
            }),
            ["store-backed execution parity belongs to Milestone 10"],
        )?)
    }

    pub fn status(&self) -> WorthQueryMilestoneClosureStatus {
        self.status
    }

    pub fn phase_closures(&self) -> &[WorthQueryMilestoneNineSevenPhaseClosure] {
        &self.phase_closures
    }

    pub fn defended_exclusions(&self) -> &[String] {
        &self.defended_exclusions
    }

    pub fn closure_digest(&self) -> &str {
        self.closure_identity.as_str()
    }

    pub fn closure_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.closure_identity
    }

    pub fn required_phases() -> &'static [&'static str] {
        &REQUIRED_PHASES
    }
}

fn derive_milestone_nine_seven_status(
    phase_closures: &[WorthQueryMilestoneNineSevenPhaseClosure],
) -> WorthQueryMilestoneClosureStatus {
    let required_closed = REQUIRED_PHASES.iter().all(|required_phase| {
        phase_closures.iter().any(|closure| {
            closure.phase() == *required_phase
                && closure.status() == WorthQueryMilestoneClosureStatus::Closed
                && !closure.evidence_digest().is_empty()
        })
    });
    if required_closed {
        return WorthQueryMilestoneClosureStatus::Closed;
    }
    if phase_closures
        .iter()
        .any(|closure| closure.status() != WorthQueryMilestoneClosureStatus::Open)
    {
        return WorthQueryMilestoneClosureStatus::Partial;
    }
    WorthQueryMilestoneClosureStatus::Open
}

#[cfg(test)]
fn milestone_status_from_pinning_posture(
    posture: WorthQuerySharedReadPinningBoundaryPosture,
) -> WorthQueryMilestoneClosureStatus {
    match posture {
        WorthQuerySharedReadPinningBoundaryPosture::Closed => {
            WorthQueryMilestoneClosureStatus::Closed
        }
        WorthQuerySharedReadPinningBoundaryPosture::Partial => {
            WorthQueryMilestoneClosureStatus::Partial
        }
        WorthQuerySharedReadPinningBoundaryPosture::Open => WorthQueryMilestoneClosureStatus::Open,
    }
}

#[cfg(test)]
fn milestone_status_from_journal_posture(
    posture: WorthQueryJournalIdentityBoundaryPosture,
) -> WorthQueryMilestoneClosureStatus {
    match posture {
        WorthQueryJournalIdentityBoundaryPosture::Closed => {
            WorthQueryMilestoneClosureStatus::Closed
        }
        WorthQueryJournalIdentityBoundaryPosture::Partial => {
            WorthQueryMilestoneClosureStatus::Partial
        }
        WorthQueryJournalIdentityBoundaryPosture::Open => WorthQueryMilestoneClosureStatus::Open,
    }
}

fn milestone_status_from_concurrent_posture(
    posture: WorthQueryConcurrentHostileMatrixPosture,
) -> WorthQueryMilestoneClosureStatus {
    match posture {
        WorthQueryConcurrentHostileMatrixPosture::Closed => {
            WorthQueryMilestoneClosureStatus::Closed
        }
        WorthQueryConcurrentHostileMatrixPosture::Partial => {
            WorthQueryMilestoneClosureStatus::Partial
        }
        WorthQueryConcurrentHostileMatrixPosture::Open => WorthQueryMilestoneClosureStatus::Open,
    }
}

fn milestone_status_from_public_bridge_posture(
    posture: WorthQueryPublicBridgeReaderLanePosture,
) -> WorthQueryMilestoneClosureStatus {
    match posture {
        WorthQueryPublicBridgeReaderLanePosture::Closed => WorthQueryMilestoneClosureStatus::Closed,
        WorthQueryPublicBridgeReaderLanePosture::Open => WorthQueryMilestoneClosureStatus::Open,
    }
}

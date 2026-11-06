use crate::{OracleFamilyKind, ShortcutRejectionObservationKind};
use worth_foundational::CanonicalDigestId;

use super::OracleVerdictBasis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalProofOracleVerdictKind {
    Satisfied,
    Denied,
    Deferred,
    Stale,
    RebindRequired,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalOracleVerdictTopologyPosture {
    ProofBacked,
    ReservedUntilProofProgression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalOracleVerdictTopology {
    kind: PhysicalProofOracleVerdictKind,
    posture: PhysicalOracleVerdictTopologyPosture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalOracleNonClaim {
    PhysicalIsolationCorrectness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalProofOracleKind {
    NoMixedRoot,
    OldReaderSeesOldRoot,
    PostSwapReaderSeesNewRoot,
    BlockedReclaimUntilRelease,
    CrashRecoversOldOrNewNeverMixed,
    NoPrivateMutation,
    NoJsonAuthority,
    CounterContract,
    TranscriptReplay,
    IndependentVerifierAgreement,
    BlobByteEquality,
    BlobChunkOrdering,
    BlobDigestChecksumDistinction,
    BlobReachability,
    BlobNoSidecarPath,
    BlobNoCrossScopeDedupe,
    BlobConstantMemory,
    BlobHeavyQualificationEvidence,
    BlobHeavyCleanup,
    BlobHeavyPatternLane,
    PhysicalIsolationInterleaving,
    IoPressureSimulation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalProofOracleVerdict {
    family: OracleFamilyKind,
    oracle: PhysicalProofOracleKind,
    kind: PhysicalProofOracleVerdictKind,
    basis: OracleVerdictBasis,
    non_claims: Vec<PhysicalOracleNonClaim>,
    transcript_replay_basis_digest: Option<CanonicalDigestId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleDenial {
    OracleFamilyNotRequired {
        family: OracleFamilyKind,
    },
    OracleFamilyMismatch {
        family: OracleFamilyKind,
        oracle: PhysicalProofOracleKind,
    },
    PlanTraceIdentityMismatch,
    MissingIndependentVerifierObservation,
    MissingRecoveryOutcomeObservation,
    MissingCheckpointInterlockObservation,
    MissingCompactionInterlockObservation,
    MissingIoPressureObservation,
    MissingBlobHarnessMetadata,
    MissingBlobHarnessObservation,
    IncompleteIoPressureAttribution,
    IoPressureEnvelopeViolated,
    CheckpointInterlockObservationDenied {
        oracle: PhysicalProofOracleKind,
    },
    CompactionInterlockObservationDenied {
        oracle: PhysicalProofOracleKind,
    },
    MissingRequiredShortcutRejectionObservation {
        required: ShortcutRejectionObservationKind,
    },
    TestSupportOracleDenied,
    LogOnlyEvidenceDenied,
    ExpectedErrorTextDenied,
    SameRunSelfComparisonDenied,
    FixtureLabelOracleDenied,
}

impl PhysicalProofOracleVerdict {
    pub(crate) fn failed(
        family: OracleFamilyKind,
        oracle: PhysicalProofOracleKind,
        basis: OracleVerdictBasis,
        non_claims: impl IntoIterator<Item = PhysicalOracleNonClaim>,
    ) -> Self {
        Self::with_kind(
            family,
            oracle,
            PhysicalProofOracleVerdictKind::Failed,
            basis,
            non_claims,
        )
    }

    pub(crate) fn satisfied(
        family: OracleFamilyKind,
        oracle: PhysicalProofOracleKind,
        basis: OracleVerdictBasis,
        non_claims: impl IntoIterator<Item = PhysicalOracleNonClaim>,
    ) -> Self {
        Self::with_kind(
            family,
            oracle,
            PhysicalProofOracleVerdictKind::Satisfied,
            basis,
            non_claims,
        )
    }

    pub(crate) fn with_transcript_replay_basis(
        mut self,
        transcript_replay_basis_digest: [u8; 32],
    ) -> Result<Self, OracleDenial> {
        if self.oracle != PhysicalProofOracleKind::TranscriptReplay {
            return Err(OracleDenial::OracleFamilyMismatch {
                family: self.family,
                oracle: self.oracle,
            });
        }
        self.transcript_replay_basis_digest =
            Some(CanonicalDigestId::new(transcript_replay_basis_digest));
        Ok(self)
    }

    fn with_kind(
        family: OracleFamilyKind,
        oracle: PhysicalProofOracleKind,
        kind: PhysicalProofOracleVerdictKind,
        basis: OracleVerdictBasis,
        non_claims: impl IntoIterator<Item = PhysicalOracleNonClaim>,
    ) -> Self {
        Self {
            family,
            oracle,
            kind,
            basis,
            non_claims: non_claims.into_iter().collect(),
            transcript_replay_basis_digest: None,
        }
    }

    pub const fn family(&self) -> OracleFamilyKind {
        self.family
    }

    pub const fn oracle(&self) -> PhysicalProofOracleKind {
        self.oracle
    }

    pub const fn kind(&self) -> PhysicalProofOracleVerdictKind {
        self.kind
    }

    pub const fn basis(&self) -> &OracleVerdictBasis {
        &self.basis
    }

    pub fn non_claims(&self) -> &[PhysicalOracleNonClaim] {
        &self.non_claims
    }

    pub const fn transcript_replay_basis_digest(&self) -> Option<&[u8; 32]> {
        match &self.transcript_replay_basis_digest {
            Some(digest) => Some(digest.bytes()),
            None => None,
        }
    }
}

impl PhysicalOracleVerdictTopology {
    pub const fn kind(&self) -> PhysicalProofOracleVerdictKind {
        self.kind
    }

    pub const fn posture(&self) -> PhysicalOracleVerdictTopologyPosture {
        self.posture
    }
}

pub const fn oracle_verdict_topology() -> [PhysicalOracleVerdictTopology; 6] {
    [
        topology(
            PhysicalProofOracleVerdictKind::Satisfied,
            PhysicalOracleVerdictTopologyPosture::ProofBacked,
        ),
        topology(
            PhysicalProofOracleVerdictKind::Denied,
            PhysicalOracleVerdictTopologyPosture::ReservedUntilProofProgression,
        ),
        topology(
            PhysicalProofOracleVerdictKind::Deferred,
            PhysicalOracleVerdictTopologyPosture::ReservedUntilProofProgression,
        ),
        topology(
            PhysicalProofOracleVerdictKind::Stale,
            PhysicalOracleVerdictTopologyPosture::ReservedUntilProofProgression,
        ),
        topology(
            PhysicalProofOracleVerdictKind::RebindRequired,
            PhysicalOracleVerdictTopologyPosture::ReservedUntilProofProgression,
        ),
        topology(
            PhysicalProofOracleVerdictKind::Failed,
            PhysicalOracleVerdictTopologyPosture::ProofBacked,
        ),
    ]
}

const fn topology(
    kind: PhysicalProofOracleVerdictKind,
    posture: PhysicalOracleVerdictTopologyPosture,
) -> PhysicalOracleVerdictTopology {
    PhysicalOracleVerdictTopology { kind, posture }
}

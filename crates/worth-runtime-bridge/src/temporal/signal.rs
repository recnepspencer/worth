use worth_foundational::facade::{CanonicalBasisSequence, CanonicalDerivedDigest};
use worth_proof::TransitionOutcome;
use worth_signal::facade::{
    ClockAdvanceOrdinal, ClockCheckpointId, ClockDomain, ClockTick, TemporalWakeId, WakeOrdinal,
};

use crate::input::envelope::TruthBranchIdentity;

use super::canonical::{
    canonical_digest, canonical_version, same_basis, text_entry, transition_canonical_ready,
    u64_entry,
};

const SIGNAL_BASIS_CANONICAL_VERSION: &str = "bridge.temporal-signal-basis.v1";
const WAKE_EVIDENCE_CANONICAL_VERSION: &str = "bridge.temporal-wake-evidence.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTemporalSignalBasis {
    branch_identity: TruthBranchIdentity,
    clock_domain: ClockDomain,
    current_tick: ClockTick,
    last_advance_ordinal: ClockAdvanceOrdinal,
    last_checkpoint_id: Option<ClockCheckpointId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTemporalWakeEvidence {
    wake_id: TemporalWakeId,
    wake_ready_ordinal: WakeOrdinal,
    wake_tick: ClockTick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeTemporalSignalBasisDenial {
    EmptyIdentityField { field: &'static str },
    WrongClockDomain { domain: ClockDomain },
}

#[derive(Debug, Clone)]
pub struct AdmittedBridgeTemporalSignalBasis {
    basis: BridgeTemporalSignalBasis,
    canonical_basis: CanonicalBasisSequence,
    canonical_digest: CanonicalDerivedDigest,
}

/// Wake evidence issued after Bridge admits a Signal promotion.
///
/// ```compile_fail,E0624
/// use worth_runtime_bridge::facade::{AdmittedBridgeTemporalWakeEvidence, BridgeTemporalWakeEvidence};
/// fn forge(wake: BridgeTemporalWakeEvidence) {
///     let _ = AdmittedBridgeTemporalWakeEvidence::admit(wake);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AdmittedBridgeTemporalWakeEvidence {
    evidence: BridgeTemporalWakeEvidence,
    canonical_basis: CanonicalBasisSequence,
    canonical_digest: CanonicalDerivedDigest,
}

impl BridgeTemporalSignalBasis {
    pub fn new(
        branch_identity: TruthBranchIdentity,
        clock_domain: ClockDomain,
        current_tick: ClockTick,
        last_advance_ordinal: ClockAdvanceOrdinal,
        last_checkpoint_id: Option<ClockCheckpointId>,
    ) -> Self {
        Self {
            branch_identity,
            clock_domain,
            current_tick,
            last_advance_ordinal,
            last_checkpoint_id,
        }
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub const fn clock_domain(&self) -> ClockDomain {
        self.clock_domain
    }

    pub const fn current_tick(&self) -> ClockTick {
        self.current_tick
    }

    pub const fn last_advance_ordinal(&self) -> ClockAdvanceOrdinal {
        self.last_advance_ordinal
    }

    pub const fn last_checkpoint_id(&self) -> Option<ClockCheckpointId> {
        self.last_checkpoint_id
    }
}

impl BridgeTemporalWakeEvidence {
    pub(crate) fn new(
        wake_id: TemporalWakeId,
        wake_ready_ordinal: WakeOrdinal,
        wake_tick: ClockTick,
    ) -> Self {
        Self {
            wake_id,
            wake_ready_ordinal,
            wake_tick,
        }
    }

    pub const fn wake_id(&self) -> TemporalWakeId {
        self.wake_id
    }

    pub const fn wake_ready_ordinal(&self) -> WakeOrdinal {
        self.wake_ready_ordinal
    }

    pub const fn wake_tick(&self) -> ClockTick {
        self.wake_tick
    }
}

impl AdmittedBridgeTemporalSignalBasis {
    pub fn admit(
        basis: BridgeTemporalSignalBasis,
    ) -> TransitionOutcome<Self, BridgeTemporalSignalBasisDenial> {
        match validate_nonempty("signal_branch", basis.branch_identity().as_str()) {
            TransitionOutcome::Success(()) => {}
            TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
            _ => unreachable!("signal identity validation uses only denied"),
        }
        if !basis.clock_domain().authority().is_authoritative() {
            return TransitionOutcome::denied(BridgeTemporalSignalBasisDenial::WrongClockDomain {
                domain: basis.clock_domain(),
            });
        }

        let canonical_ready = transition_canonical_ready(
            canonical_version(SIGNAL_BASIS_CANONICAL_VERSION),
            [
                text_entry("signal_branch", basis.branch_identity().as_str()),
                text_entry(
                    "signal_clock_domain",
                    clock_domain_label(basis.clock_domain()),
                ),
                u64_entry("signal_clock_tick", basis.current_tick().get()),
                u64_entry(
                    "signal_clock_advance_ordinal",
                    basis.last_advance_ordinal().get(),
                ),
                optional_checkpoint_entry(basis.last_checkpoint_id()),
            ],
            "temporal signal basis canonicalization denied",
        );
        let canonical_basis = canonical_ready.payload().clone();
        let canonical_digest = canonical_digest(
            canonical_ready,
            "temporal signal basis digest admission denied",
        );

        TransitionOutcome::success(Self {
            basis,
            canonical_basis,
            canonical_digest,
        })
    }

    pub fn basis(&self) -> &BridgeTemporalSignalBasis {
        &self.basis
    }

    pub fn canonical_basis(&self) -> &CanonicalBasisSequence {
        &self.canonical_basis
    }

    pub fn canonical_digest(&self) -> &CanonicalDerivedDigest {
        &self.canonical_digest
    }
}

impl PartialEq for AdmittedBridgeTemporalSignalBasis {
    fn eq(&self, other: &Self) -> bool {
        self.basis == other.basis
            && same_basis(&self.canonical_basis, &other.canonical_basis)
            && self.canonical_digest == other.canonical_digest
    }
}

impl Eq for AdmittedBridgeTemporalSignalBasis {}

impl AdmittedBridgeTemporalWakeEvidence {
    pub(crate) fn admit(evidence: BridgeTemporalWakeEvidence) -> Self {
        let canonical_ready = transition_canonical_ready(
            canonical_version(WAKE_EVIDENCE_CANONICAL_VERSION),
            [
                u64_entry("wake_id", evidence.wake_id().get()),
                u64_entry("wake_ready_ordinal", evidence.wake_ready_ordinal().get()),
                u64_entry("wake_tick", evidence.wake_tick().get()),
            ],
            "temporal wake evidence canonicalization denied",
        );
        let canonical_basis = canonical_ready.payload().clone();
        let canonical_digest = canonical_digest(
            canonical_ready,
            "temporal wake evidence digest admission denied",
        );

        Self {
            evidence,
            canonical_basis,
            canonical_digest,
        }
    }

    pub fn evidence(&self) -> &BridgeTemporalWakeEvidence {
        &self.evidence
    }

    pub fn canonical_basis(&self) -> &CanonicalBasisSequence {
        &self.canonical_basis
    }

    pub fn canonical_digest(&self) -> &CanonicalDerivedDigest {
        &self.canonical_digest
    }
}

impl PartialEq for AdmittedBridgeTemporalWakeEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.evidence == other.evidence
            && same_basis(&self.canonical_basis, &other.canonical_basis)
            && self.canonical_digest == other.canonical_digest
    }
}

impl Eq for AdmittedBridgeTemporalWakeEvidence {}

fn validate_nonempty<Denial>(field: &'static str, value: &str) -> TransitionOutcome<(), Denial>
where
    Denial: FromEmptyField,
{
    if value.trim().is_empty() {
        TransitionOutcome::denied(Denial::from_empty_field(field))
    } else {
        TransitionOutcome::success(())
    }
}

trait FromEmptyField {
    fn from_empty_field(field: &'static str) -> Self;
}

impl FromEmptyField for BridgeTemporalSignalBasisDenial {
    fn from_empty_field(field: &'static str) -> Self {
        Self::EmptyIdentityField { field }
    }
}

fn clock_domain_label(domain: ClockDomain) -> &'static str {
    match domain {
        ClockDomain::MonotonicExecution => "monotonic_execution",
        ClockDomain::WallClock => "wall_clock",
        ClockDomain::Presentation => "presentation",
    }
}

fn optional_checkpoint_entry(
    checkpoint_id: Option<ClockCheckpointId>,
) -> worth_foundational::facade::CanonicalBasisEntry {
    match checkpoint_id {
        Some(checkpoint_id) => u64_entry("signal_clock_checkpoint", checkpoint_id.get()),
        None => text_entry("signal_clock_checkpoint", "none"),
    }
}

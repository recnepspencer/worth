use worth_foundational::facade::{CanonicalBasisSequence, CanonicalDerivedDigest};
use worth_proof::TransitionOutcome;

use crate::canonical_basis::canonical_basis_ready_text;
use crate::identity::{BridgeIdentity, TemporalBasisIdentityTag, TemporalCdcCursorIdentityTag};
use crate::input::envelope::TruthBranchIdentity;

use super::basis_kind::BridgeTemporalBasisKind;
use super::canonical::{
    canonical_digest, canonical_version, hex_bytes, rebuild_ready, same_basis, text_entry,
    transition_canonical_ready,
};
use super::signal::{
    AdmittedBridgeTemporalSignalBasis, AdmittedBridgeTemporalWakeEvidence,
    BridgeTemporalSignalBasis, BridgeTemporalSignalBasisDenial, BridgeTemporalWakeEvidence,
};
use super::truth::{
    AdmittedBridgeTemporalTruthViewBasis, BridgeTemporalTruthBasisDenial,
    BridgeTemporalTruthViewBasis,
};

const TEMPORAL_BASIS_CANONICAL_VERSION: &str = "bridge.temporal-basis.v1";

pub type BridgeTemporalBasisIdentity = BridgeIdentity<TemporalBasisIdentityTag>;
pub type BridgeTemporalCdcCursorIdentity = BridgeIdentity<TemporalCdcCursorIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeTemporalBasisDenial {
    TruthBasisDenied(BridgeTemporalTruthBasisDenial),
    SignalBasisDenied(BridgeTemporalSignalBasisDenial),
    MissingWakeEvidence,
    BranchMismatch {
        truth_branch_identity: TruthBranchIdentity,
        signal_branch_identity: TruthBranchIdentity,
    },
    WakeTickRegressed {
        signal_clock_tick: u64,
        wake_tick: u64,
    },
}

#[derive(Debug, Clone)]
pub struct AdmittedBridgeTemporalBasis {
    identity: BridgeTemporalBasisIdentity,
    kind: BridgeTemporalBasisKind,
    truth_basis: AdmittedBridgeTemporalTruthViewBasis,
    signal_basis: AdmittedBridgeTemporalSignalBasis,
    wake_evidence: AdmittedBridgeTemporalWakeEvidence,
    canonical_basis: CanonicalBasisSequence,
    canonical_digest: CanonicalDerivedDigest,
}

impl AdmittedBridgeTemporalBasis {
    pub(crate) fn admit(
        truth_basis: BridgeTemporalTruthViewBasis,
        signal_basis: BridgeTemporalSignalBasis,
        wake_evidence: Option<BridgeTemporalWakeEvidence>,
    ) -> TransitionOutcome<Self, BridgeTemporalBasisDenial> {
        let truth_basis = match AdmittedBridgeTemporalTruthViewBasis::admit(truth_basis) {
            TransitionOutcome::Success(admitted) => admitted,
            TransitionOutcome::Denied(denial) => {
                return TransitionOutcome::Denied(BridgeTemporalBasisDenial::TruthBasisDenied(
                    denial,
                ));
            }
            _ => unreachable!("truth basis admission uses only denied"),
        };
        let signal_basis = match AdmittedBridgeTemporalSignalBasis::admit(signal_basis) {
            TransitionOutcome::Success(admitted) => admitted,
            TransitionOutcome::Denied(denial) => {
                return TransitionOutcome::Denied(BridgeTemporalBasisDenial::SignalBasisDenied(
                    denial,
                ));
            }
            _ => unreachable!("signal basis admission uses only denied"),
        };
        let Some(wake_evidence) = wake_evidence else {
            return TransitionOutcome::denied(BridgeTemporalBasisDenial::MissingWakeEvidence);
        };

        if truth_basis.basis().branch_identity() != signal_basis.basis().branch_identity() {
            return TransitionOutcome::denied(BridgeTemporalBasisDenial::BranchMismatch {
                truth_branch_identity: truth_basis.basis().branch_identity().clone(),
                signal_branch_identity: signal_basis.basis().branch_identity().clone(),
            });
        }

        if wake_evidence.wake_tick() < signal_basis.basis().current_tick() {
            return TransitionOutcome::denied(BridgeTemporalBasisDenial::WakeTickRegressed {
                signal_clock_tick: signal_basis.basis().current_tick().get(),
                wake_tick: wake_evidence.wake_tick().get(),
            });
        }

        let wake_evidence = AdmittedBridgeTemporalWakeEvidence::admit(wake_evidence);
        let canonical_ready = transition_canonical_ready(
            canonical_version(TEMPORAL_BASIS_CANONICAL_VERSION),
            [
                text_entry(
                    "truth_basis_digest",
                    &digest_text(truth_basis.canonical_digest()),
                ),
                text_entry(
                    "signal_basis_digest",
                    &digest_text(signal_basis.canonical_digest()),
                ),
                text_entry(
                    "wake_evidence_digest",
                    &digest_text(wake_evidence.canonical_digest()),
                ),
            ],
            "temporal bridge basis canonicalization denied",
        );
        let canonical_basis = canonical_ready.payload().clone();
        let canonical_digest = canonical_digest(
            canonical_ready,
            "temporal bridge basis digest admission denied",
        );
        let identity = BridgeTemporalBasisIdentity::admit_bridge_owned(hex_bytes(
            canonical_digest.value().bytes(),
        ));

        TransitionOutcome::success(Self {
            identity,
            kind: truth_basis.basis().kind(),
            truth_basis,
            signal_basis,
            wake_evidence,
            canonical_basis,
            canonical_digest,
        })
    }

    pub fn identity(&self) -> &BridgeTemporalBasisIdentity {
        &self.identity
    }

    pub const fn kind(&self) -> BridgeTemporalBasisKind {
        self.kind
    }

    pub fn truth_basis(&self) -> &AdmittedBridgeTemporalTruthViewBasis {
        &self.truth_basis
    }

    pub fn signal_basis(&self) -> &AdmittedBridgeTemporalSignalBasis {
        &self.signal_basis
    }

    pub fn wake_evidence(&self) -> &AdmittedBridgeTemporalWakeEvidence {
        &self.wake_evidence
    }

    pub fn canonical_basis(&self) -> &CanonicalBasisSequence {
        &self.canonical_basis
    }

    pub fn canonical_digest(&self) -> &CanonicalDerivedDigest {
        &self.canonical_digest
    }

    pub fn canonical_basis_text(&self) -> String {
        canonical_basis_ready_text(&rebuild_ready(&self.canonical_basis))
            .expect("temporal basis canonical text stays renderable")
    }
}

impl PartialEq for AdmittedBridgeTemporalBasis {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.kind == other.kind
            && self.truth_basis == other.truth_basis
            && self.signal_basis == other.signal_basis
            && self.wake_evidence == other.wake_evidence
            && same_basis(&self.canonical_basis, &other.canonical_basis)
            && self.canonical_digest == other.canonical_digest
    }
}

impl Eq for AdmittedBridgeTemporalBasis {}

fn digest_text(digest: &CanonicalDerivedDigest) -> String {
    hex_bytes(digest.value().bytes())
}

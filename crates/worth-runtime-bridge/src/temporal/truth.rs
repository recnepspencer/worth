use worth_foundational::facade::{CanonicalBasisSequence, CanonicalDerivedDigest};
use worth_proof::TransitionOutcome;

use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::snapshot::TruthSnapshotIdentity;

use super::basis::BridgeTemporalCdcCursorIdentity;
use super::basis_kind::BridgeTemporalBasisKind;
use super::canonical::{
    canonical_digest, canonical_version, same_basis, text_entry, transition_canonical_ready,
};

const TRUTH_BASIS_CANONICAL_VERSION: &str = "bridge.temporal-truth-basis.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeTemporalTruthViewBasis {
    Authoritative {
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    },
    BranchHead {
        branch_identity: TruthBranchIdentity,
        head_commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    },
    Historical {
        branch_identity: TruthBranchIdentity,
        pinned_commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    },
    CdcCursor {
        branch_identity: TruthBranchIdentity,
        cursor_identity: BridgeTemporalCdcCursorIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeTemporalTruthBasisDenial {
    EmptyIdentityField { field: &'static str },
}

#[derive(Debug, Clone)]
pub struct AdmittedBridgeTemporalTruthViewBasis {
    basis: BridgeTemporalTruthViewBasis,
    canonical_basis: CanonicalBasisSequence,
    canonical_digest: CanonicalDerivedDigest,
}

impl BridgeTemporalTruthViewBasis {
    pub fn authoritative(
        branch_identity: TruthBranchIdentity,
        commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::Authoritative {
            branch_identity,
            commit_identity,
            snapshot_identity,
        }
    }

    pub fn branch_head(
        branch_identity: TruthBranchIdentity,
        head_commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::BranchHead {
            branch_identity,
            head_commit_identity,
            snapshot_identity,
        }
    }

    pub fn historical(
        branch_identity: TruthBranchIdentity,
        pinned_commit_identity: TruthCommitIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::Historical {
            branch_identity,
            pinned_commit_identity,
            snapshot_identity,
        }
    }

    pub fn cdc_cursor(
        branch_identity: TruthBranchIdentity,
        cursor_identity: BridgeTemporalCdcCursorIdentity,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        Self::CdcCursor {
            branch_identity,
            cursor_identity,
            snapshot_identity,
        }
    }

    pub const fn kind(&self) -> BridgeTemporalBasisKind {
        match self {
            Self::Authoritative { .. } => BridgeTemporalBasisKind::Authoritative,
            Self::BranchHead { .. } => BridgeTemporalBasisKind::BranchHead,
            Self::Historical { .. } => BridgeTemporalBasisKind::Historical,
            Self::CdcCursor { .. } => BridgeTemporalBasisKind::CdcCursor,
        }
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        match self {
            Self::Authoritative {
                branch_identity, ..
            }
            | Self::BranchHead {
                branch_identity, ..
            }
            | Self::Historical {
                branch_identity, ..
            }
            | Self::CdcCursor {
                branch_identity, ..
            } => branch_identity,
        }
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        match self {
            Self::Authoritative {
                snapshot_identity, ..
            }
            | Self::BranchHead {
                snapshot_identity, ..
            }
            | Self::Historical {
                snapshot_identity, ..
            }
            | Self::CdcCursor {
                snapshot_identity, ..
            } => snapshot_identity,
        }
    }

    pub fn native_truth_locator(&self) -> String {
        match self {
            Self::Authoritative {
                commit_identity, ..
            } => commit_identity.canonical_label(),
            Self::BranchHead {
                head_commit_identity,
                ..
            } => head_commit_identity.canonical_label(),
            Self::Historical {
                pinned_commit_identity,
                ..
            } => pinned_commit_identity.canonical_label(),
            Self::CdcCursor {
                cursor_identity, ..
            } => cursor_identity.as_str().to_owned(),
        }
    }
}

impl AdmittedBridgeTemporalTruthViewBasis {
    pub fn admit(
        basis: BridgeTemporalTruthViewBasis,
    ) -> TransitionOutcome<Self, BridgeTemporalTruthBasisDenial> {
        let branch = basis.branch_identity();
        let branch_label = branch.canonical_label();
        let branch_name = branch.relational_branch_id().unwrap_or(&branch_label);
        match validate_nonempty("truth_branch", branch_name) {
            TransitionOutcome::Success(()) => {}
            TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
            _ => unreachable!("truth identity validation uses only denied"),
        }
        let locator = basis.native_truth_locator();
        if matches!(basis, BridgeTemporalTruthViewBasis::CdcCursor { .. }) {
            match validate_nonempty("truth_locator", &locator) {
                TransitionOutcome::Success(()) => {}
                TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
                _ => unreachable!("truth identity validation uses only denied"),
            }
        }

        let canonical_ready = transition_canonical_ready(
            canonical_version(TRUTH_BASIS_CANONICAL_VERSION),
            [
                text_entry("truth_kind", basis.kind().canonical_label()),
                text_entry("truth_branch", &branch_label),
                text_entry(
                    "truth_snapshot",
                    &basis.snapshot_identity().canonical_label(),
                ),
                text_entry("truth_locator", &locator),
            ],
            "temporal truth basis canonicalization denied",
        );
        let canonical_basis = canonical_ready.payload().clone();
        let canonical_digest = canonical_digest(
            canonical_ready,
            "temporal truth basis digest admission denied",
        );

        TransitionOutcome::success(Self {
            basis,
            canonical_basis,
            canonical_digest,
        })
    }

    pub fn basis(&self) -> &BridgeTemporalTruthViewBasis {
        &self.basis
    }

    pub fn canonical_basis(&self) -> &CanonicalBasisSequence {
        &self.canonical_basis
    }

    pub fn canonical_digest(&self) -> &CanonicalDerivedDigest {
        &self.canonical_digest
    }
}

impl PartialEq for AdmittedBridgeTemporalTruthViewBasis {
    fn eq(&self, other: &Self) -> bool {
        self.basis == other.basis
            && same_basis(&self.canonical_basis, &other.canonical_basis)
            && self.canonical_digest == other.canonical_digest
    }
}

impl Eq for AdmittedBridgeTemporalTruthViewBasis {}

fn validate_nonempty(
    field: &'static str,
    value: &str,
) -> TransitionOutcome<(), BridgeTemporalTruthBasisDenial> {
    if value.trim().is_empty() {
        TransitionOutcome::denied(BridgeTemporalTruthBasisDenial::EmptyIdentityField { field })
    } else {
        TransitionOutcome::success(())
    }
}

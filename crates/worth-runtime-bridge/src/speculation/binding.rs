use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{
    BridgeIdentity, BridgeIdentityEvidence, PreviewBranchBindingIdentityTag,
    SpeculativeSignalBranchIdentityTag,
};
use crate::input::envelope::TruthBranchIdentity;

pub type BridgeSignalBranchIdentity = BridgeIdentity<SpeculativeSignalBranchIdentityTag>;
pub type BridgeSpeculativeBranchBindingIdentity = BridgeIdentity<PreviewBranchBindingIdentityTag>;

impl BridgeSignalBranchIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-preview-signal-branch:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }

    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

impl BridgeSpeculativeBranchBindingIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-preview-branch-binding:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }

    pub fn from_stable_name(value: impl Into<Arc<str>>) -> Self {
        Self::admit_bridge_owned(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalForkAxis {
    SharedMain,
    Forked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalForkAxis {
    SharedMain,
    Forked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchForkPosture {
    pub relational: RelationalForkAxis,
    pub signal: SignalForkAxis,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSpeculativeBranchBinding {
    binding_identity: BridgeSpeculativeBranchBindingIdentity,
    truth_branch_identity: TruthBranchIdentity,
    signal_branch_identity: BridgeSignalBranchIdentity,
    fork_posture: BranchForkPosture,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSpeculativeBranchBinding {
    pub fn new(
        binding_identity: BridgeSpeculativeBranchBindingIdentity,
        truth_branch_identity: TruthBranchIdentity,
        signal_branch_identity: BridgeSignalBranchIdentity,
    ) -> Self {
        Self::with_fork_posture(
            binding_identity,
            truth_branch_identity,
            signal_branch_identity,
            BranchForkPosture {
                relational: RelationalForkAxis::Forked,
                signal: SignalForkAxis::Forked,
            },
        )
    }

    pub fn with_fork_posture(
        binding_identity: BridgeSpeculativeBranchBindingIdentity,
        truth_branch_identity: TruthBranchIdentity,
        signal_branch_identity: BridgeSignalBranchIdentity,
        fork_posture: BranchForkPosture,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "speculative-branch-binding|id={}|truth-branch={}|signal-branch={}|relational-fork={:?}|signal-fork={:?}",
            binding_identity.as_str(),
            truth_branch_identity.as_str(),
            signal_branch_identity.as_str(),
            fork_posture.relational,
            fork_posture.signal,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            binding_identity,
            truth_branch_identity,
            signal_branch_identity,
            fork_posture,
            canonical_basis,
            digest: Arc::from(format!("speculative-branch-binding:sha256:{digest:x}")),
        }
    }

    pub fn binding_identity(&self) -> &BridgeSpeculativeBranchBindingIdentity {
        &self.binding_identity
    }

    pub fn truth_branch_identity(&self) -> &TruthBranchIdentity {
        &self.truth_branch_identity
    }

    pub fn signal_branch_identity(&self) -> &BridgeSignalBranchIdentity {
        &self.signal_branch_identity
    }

    pub fn fork_posture(&self) -> BranchForkPosture {
        self.fork_posture
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

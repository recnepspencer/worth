mod retained_charge;

use serde::{Deserialize, Serialize};
use worth_proof::{
    Artifact, AssumptionBasis, AuthorityMarker, AuthorityWitness,
    BoundaryBridgedAuthorityRevalidationRequiredBasis, CurrentValidity, FreshnessScopedBasis,
    NoProofs, PhaseMarker,
};

use crate::logic::transaction::runtime::state::merge::proof::canonical_digest;
use crate::logic::transaction::runtime::state::SignalBranchBasisIdentity;

use super::facts::SignalMergeCompatibilityFactInventory;

pub const SIGNAL_MERGE_COMPATIBILITY_SCHEMA_VERSION: &str = "worth-signal-merge-compat-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalMergeCompatibilityReady;

impl PhaseMarker for SignalMergeCompatibilityReady {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalMergeCompatibilityAuthority(());

impl SignalMergeCompatibilityAuthority {
    pub(crate) const fn new() -> Self {
        Self(())
    }
}

impl AuthorityMarker for SignalMergeCompatibilityAuthority {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalMergeCompatibilityReadmissionAuthority(());

impl SignalMergeCompatibilityReadmissionAuthority {
    pub(crate) const fn new() -> Self {
        Self(())
    }
}

impl AuthorityMarker for SignalMergeCompatibilityReadmissionAuthority {}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalMergeCompatibilityPostureKind {
    CurrentBasis,
    BoundaryBridgedAuthorityRevalidationRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalMergeCompatibilityBasis {
    branch_basis_identity: SignalBranchBasisIdentity,
    declaration_digest: String,
    admitted_scope_digest: String,
    strategy_witness_digest: String,
}

impl SignalMergeCompatibilityBasis {
    pub(crate) fn new(
        branch_basis_identity: SignalBranchBasisIdentity,
        declaration_digest: String,
        admitted_scope_digest: String,
        strategy_witness_digest: String,
    ) -> Self {
        Self {
            branch_basis_identity,
            declaration_digest,
            admitted_scope_digest,
            strategy_witness_digest,
        }
    }

    pub fn branch_basis_identity(&self) -> &SignalBranchBasisIdentity {
        &self.branch_basis_identity
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn admitted_scope_digest(&self) -> &str {
        &self.admitted_scope_digest
    }

    pub fn strategy_witness_digest(&self) -> &str {
        &self.strategy_witness_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalMergeCompatibilityWitness {
    schema_version: String,
    fact_inventory: SignalMergeCompatibilityFactInventory,
    compatibility_digest: String,
}

impl SignalMergeCompatibilityWitness {
    pub(crate) fn new(fact_inventory: SignalMergeCompatibilityFactInventory) -> Self {
        let compatibility_digest = canonical_digest(&fact_inventory);
        Self {
            schema_version: SIGNAL_MERGE_COMPATIBILITY_SCHEMA_VERSION.to_owned(),
            fact_inventory,
            compatibility_digest,
        }
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn fact_inventory(&self) -> &SignalMergeCompatibilityFactInventory {
        &self.fact_inventory
    }

    pub fn compatibility_digest(&self) -> &str {
        &self.compatibility_digest
    }

    pub(crate) fn try_from_replay_record(
        schema_version: String,
        fact_inventory: SignalMergeCompatibilityFactInventory,
        compatibility_digest: String,
    ) -> Result<Self, String> {
        if schema_version != SIGNAL_MERGE_COMPATIBILITY_SCHEMA_VERSION {
            return Err(format!(
                "unexpected compatibility witness schema version during replay decode: {schema_version}"
            ));
        }
        let expected_digest = canonical_digest(&fact_inventory);
        if compatibility_digest != expected_digest {
            return Err(format!(
                "compatibility witness digest mismatch during replay decode: expected {expected_digest}, observed {compatibility_digest}"
            ));
        }
        Ok(Self {
            schema_version,
            fact_inventory,
            compatibility_digest,
        })
    }
}

pub type SignalMergeCompatibilityArtifact = Artifact<
    SignalMergeCompatibilityReady,
    SignalMergeCompatibilityWitness,
    NoProofs,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<SignalMergeCompatibilityBasis>>,
>;

pub type BoundaryBridgedSignalMergeCompatibilityArtifact = Artifact<
    SignalMergeCompatibilityReady,
    SignalMergeCompatibilityWitness,
    NoProofs,
    BoundaryBridgedAuthorityRevalidationRequiredBasis<SignalMergeCompatibilityBasis>,
>;

pub fn new_signal_merge_compatibility_artifact(
    payload: SignalMergeCompatibilityWitness,
    basis: SignalMergeCompatibilityBasis,
) -> SignalMergeCompatibilityArtifact {
    Artifact::with_current_basis(
        payload,
        basis,
        AuthorityWitness::from_authority_marker(SignalMergeCompatibilityAuthority::new()),
    )
}

#[allow(dead_code)]
pub fn bridge_signal_merge_compatibility_trust_boundary(
    artifact: SignalMergeCompatibilityArtifact,
) -> BoundaryBridgedSignalMergeCompatibilityArtifact {
    artifact.bridge_trust_boundary()
}

#[allow(dead_code)]
pub fn compatibility_posture_kind(
    _artifact: &SignalMergeCompatibilityArtifact,
) -> SignalMergeCompatibilityPostureKind {
    SignalMergeCompatibilityPostureKind::CurrentBasis
}

#[allow(dead_code)]
pub fn bridged_compatibility_posture_kind(
    _artifact: &BoundaryBridgedSignalMergeCompatibilityArtifact,
) -> SignalMergeCompatibilityPostureKind {
    SignalMergeCompatibilityPostureKind::BoundaryBridgedAuthorityRevalidationRequired
}

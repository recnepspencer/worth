mod retained_charge;

use std::convert::Infallible;

use serde::{Deserialize, Serialize};
use worth_proof::{
    Artifact, AssumptionBasis, BoundaryBridgedAuthorityRevalidationRequiredBasis, CurrentValidity,
    FreshnessScopedBasis, PhaseMarker, StaleReadableBasis, TransitionOutcome,
};

use crate::branch::{
    mint_signal_branch_authority, signal_branch_basis_proof, SignalBranchBasisAuthority,
    SignalBranchBasisProof,
};
use crate::logic::transaction::canonical_digest;
use crate::state::{
    SignalBranchHandle, SignalBranchId, SignalSnapshotId, SignalSnapshotV1, SnapshotRestoreIntent,
};

pub const SIGNAL_BRANCH_BASIS_SCHEMA_VERSION: &str = "worth-signal-branch-basis-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalBranchBasisReady;

impl PhaseMarker for SignalBranchBasisReady {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchHeadPosture {
    NoHeadSnapshot,
    HeadSnapshot(SignalSnapshotId),
}

impl SignalBranchHeadPosture {
    fn from_snapshot(snapshot_id: Option<SignalSnapshotId>) -> Self {
        snapshot_id
            .map(Self::HeadSnapshot)
            .unwrap_or(Self::NoHeadSnapshot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRestorePosture {
    NotRestoreDerived,
    SnapshotRestore {
        snapshot_id: SignalSnapshotId,
        intent: SnapshotRestoreIntent,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchBasisIdentity {
    pub branch_id: SignalBranchId,
    pub snapshot_id: Option<SignalSnapshotId>,
    pub head_posture: SignalBranchHeadPosture,
    pub restore_posture: SignalBranchRestorePosture,
}

impl SignalBranchBasisIdentity {
    /// Lower only the immutable target basis. The branch identity remains an
    /// observation axis and must be supplied to the signal observation adapter
    /// from this identity's owner-issued branch_id.
    pub fn to_foundational_target(
        &self,
        graph_instance_id: impl Into<String>,
        definition_basis: u64,
    ) -> Result<
        crate::branch::SignalBranchTarget,
        crate::branch::SignalBranchTargetConstructionDenial,
    > {
        let restore_snapshot_id = match &self.restore_posture {
            SignalBranchRestorePosture::NotRestoreDerived => None,
            SignalBranchRestorePosture::SnapshotRestore { snapshot_id, .. } => Some(snapshot_id.0),
        };
        crate::branch::SignalBranchTarget::new(
            graph_instance_id,
            definition_basis,
            self.snapshot_id.map(|snapshot_id| snapshot_id.0),
            restore_snapshot_id,
        )
    }

    /// Lower this owner-issued identity into a complete shared observation.
    /// The branch id is carried from the identity rather than guessed from
    /// the human-readable branch name.
    pub fn to_foundational_observation(
        &self,
        graph_instance_id: impl AsRef<str>,
        branch_name: impl AsRef<str>,
        definition_basis: u64,
        generation: worth_foundational::FoundationalBranchReferenceGeneration,
    ) -> Result<
        crate::branch::SignalBranchObservation,
        crate::branch::SignalBranchObservationConstructionDenial,
    > {
        let target = self.to_foundational_target(graph_instance_id.as_ref(), definition_basis)?;
        crate::branch::signal_branch_observation(
            graph_instance_id,
            self.branch_id.0,
            branch_name,
            worth_foundational::FoundationalBranchTarget::basis(target),
            generation,
        )
    }

    pub(super) fn from_branch_handle(branch: &SignalBranchHandle) -> Self {
        Self::from_branch_handle_with_restore(branch, None)
    }

    pub(super) fn from_branch_handle_with_restore(
        branch: &SignalBranchHandle,
        restore_snapshot_id: Option<SignalSnapshotId>,
    ) -> Self {
        Self {
            branch_id: branch.id,
            snapshot_id: branch.head_snapshot_id,
            head_posture: SignalBranchHeadPosture::from_snapshot(branch.head_snapshot_id),
            restore_posture: restore_snapshot_id
                .map(|snapshot_id| SignalBranchRestorePosture::SnapshotRestore {
                    snapshot_id,
                    intent: SnapshotRestoreIntent::restore_runtime_truth(),
                })
                .unwrap_or(SignalBranchRestorePosture::NotRestoreDerived),
        }
    }

    pub(super) fn from_snapshot_restore(
        snapshot: &SignalSnapshotV1,
        intent: SnapshotRestoreIntent,
    ) -> Self {
        Self {
            branch_id: snapshot.meta.branch_id,
            snapshot_id: Some(snapshot.meta.snapshot_id),
            head_posture: SignalBranchHeadPosture::HeadSnapshot(snapshot.meta.snapshot_id),
            restore_posture: SignalBranchRestorePosture::SnapshotRestore {
                snapshot_id: snapshot.meta.snapshot_id,
                intent,
            },
        }
    }

    pub(super) fn from_branch_snapshot(
        branch: &SignalBranchHandle,
        snapshot_id: SignalSnapshotId,
    ) -> Self {
        Self {
            branch_id: branch.id,
            snapshot_id: Some(snapshot_id),
            head_posture: SignalBranchHeadPosture::HeadSnapshot(snapshot_id),
            restore_posture: SignalBranchRestorePosture::NotRestoreDerived,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchBasis {
    schema_version: String,
    branch_id: SignalBranchId,
    branch_name: String,
    snapshot_id: Option<SignalSnapshotId>,
    head_posture: SignalBranchHeadPosture,
    restore_posture: SignalBranchRestorePosture,
    branch_component_digest: String,
    snapshot_component_digest: String,
    head_component_digest: String,
    restore_component_digest: String,
    basis_digest: String,
}

impl SignalBranchBasis {
    fn from_identity(branch_name: impl Into<String>, identity: SignalBranchBasisIdentity) -> Self {
        let branch_component_digest = canonical_digest(&identity.branch_id.0);
        let snapshot_component_digest =
            canonical_digest(&identity.snapshot_id.map(|snapshot| snapshot.0));
        let head_component_digest = canonical_digest(&identity.head_posture);
        let restore_component_digest = canonical_digest(&identity.restore_posture);
        let basis_digest = canonical_digest(&identity);
        Self {
            schema_version: SIGNAL_BRANCH_BASIS_SCHEMA_VERSION.to_owned(),
            branch_id: identity.branch_id,
            branch_name: branch_name.into(),
            snapshot_id: identity.snapshot_id,
            head_posture: identity.head_posture,
            restore_posture: identity.restore_posture,
            branch_component_digest,
            snapshot_component_digest,
            head_component_digest,
            restore_component_digest,
            basis_digest,
        }
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub fn branch_name(&self) -> &str {
        &self.branch_name
    }

    pub fn snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.snapshot_id
    }

    pub fn head_posture(&self) -> &SignalBranchHeadPosture {
        &self.head_posture
    }

    pub fn restore_posture(&self) -> &SignalBranchRestorePosture {
        &self.restore_posture
    }

    pub fn branch_component_digest(&self) -> &str {
        &self.branch_component_digest
    }

    pub fn snapshot_component_digest(&self) -> &str {
        &self.snapshot_component_digest
    }

    pub fn head_component_digest(&self) -> &str {
        &self.head_component_digest
    }

    pub fn restore_component_digest(&self) -> &str {
        &self.restore_component_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchBasisDenial {
    UnknownBranch {
        branch_id: SignalBranchId,
        branch_name: String,
    },
    UntrackedSnapshot {
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
    CrossBranchMismatch {
        basis_branch_id: SignalBranchId,
        expected_branch_id: SignalBranchId,
    },
}

pub type SignalBranchBasisArtifact = Artifact<
    SignalBranchBasisReady,
    SignalBranchBasis,
    SignalBranchBasisProof,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<SignalBranchBasisIdentity>>,
>;

type BoundaryBridgedSignalBranchBasisState = Artifact<
    SignalBranchBasisReady,
    SignalBranchBasis,
    SignalBranchBasisProof,
    BoundaryBridgedAuthorityRevalidationRequiredBasis<SignalBranchBasisIdentity>,
>;

/// Boundary-weakened branch basis with an owner-specific readmission door.
///
/// The underlying Proof artifact remains private so the public surface cannot
/// expose `Artifact::readmit_with_authority<Auth: AuthorityMarker>` as a
/// generic authority lane. Signal readmission must use Signal's concrete
/// authority witness.
pub struct BoundaryBridgedSignalBranchBasisArtifact(BoundaryBridgedSignalBranchBasisState);

impl BoundaryBridgedSignalBranchBasisArtifact {
    pub fn readmit_with_authority(
        self,
        basis: SignalBranchBasisIdentity,
        authority: SignalBranchBasisAuthority,
    ) -> SignalBranchBasisArtifact {
        self.0.readmit_with_authority(basis, authority)
    }
}

pub type StaleSignalBranchBasisArtifact = Artifact<
    SignalBranchBasisReady,
    SignalBranchBasis,
    SignalBranchBasisProof,
    StaleReadableBasis<SignalBranchBasisIdentity>,
>;

pub type SignalBranchBasisValidationOutcome = TransitionOutcome<
    SignalBranchBasisArtifact,
    SignalBranchBasisDenial,
    Infallible,
    StaleSignalBranchBasisArtifact,
>;

pub(super) fn materialize_branch_basis(
    branch_name: impl Into<String>,
    identity: SignalBranchBasisIdentity,
) -> SignalBranchBasisArtifact {
    let payload = SignalBranchBasis::from_identity(branch_name, identity.clone());
    let authority = mint_signal_branch_authority();
    let proofs = signal_branch_basis_proof(&authority);
    Artifact::with_proofs_and_current_basis(payload, proofs, identity, authority)
}

pub fn bridge_signal_branch_basis_trust_boundary(
    basis: SignalBranchBasisArtifact,
) -> BoundaryBridgedSignalBranchBasisArtifact {
    BoundaryBridgedSignalBranchBasisArtifact(basis.bridge_trust_boundary())
}

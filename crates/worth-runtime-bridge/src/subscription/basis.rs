use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::delivery::open_planned_snapshot;
use crate::error::{BridgeDeliveryErrorKind, BridgeTypedError};
use crate::facade::RuntimeBridge;
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::snapshot::TruthSnapshotIdentity;

use super::{
    BridgeSubscriptionBasisIdentity, BridgeSubscriptionCounters, BridgeSubscriptionDeclaration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeSubscriptionBasisKind {
    Snapshot,
    BranchHead,
}

impl BridgeSubscriptionBasisKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Snapshot => "snapshot",
            Self::BranchHead => "branch_head",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionBasisResolutionFailureKind {
    MissingBranchHeadSource,
    BranchHeadResolutionFailure,
    BranchHeadMismatch,
    SnapshotAcquisitionFailure,
    SnapshotIdentityMismatch,
}

impl BridgeSubscriptionBasisResolutionFailureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBranchHeadSource => "missing_branch_head_source",
            Self::BranchHeadResolutionFailure => "branch_head_resolution_failure",
            Self::BranchHeadMismatch => "branch_head_mismatch",
            Self::SnapshotAcquisitionFailure => "snapshot_acquisition_failure",
            Self::SnapshotIdentityMismatch => "snapshot_identity_mismatch",
        }
    }
}

pub type BridgeSubscriptionBasisResolutionFailure =
    BridgeTypedError<BridgeSubscriptionBasisResolutionFailureKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeSubscriptionBasisRequest {
    Snapshot {
        snapshot_identity: TruthSnapshotIdentity,
    },
    BranchHead {
        branch_identity: TruthBranchIdentity,
    },
}

impl BridgeSubscriptionBasisRequest {
    pub fn snapshot(snapshot_identity: TruthSnapshotIdentity) -> Self {
        Self::Snapshot { snapshot_identity }
    }

    pub fn branch_head(branch_identity: TruthBranchIdentity) -> Self {
        Self::BranchHead { branch_identity }
    }

    pub fn basis_kind(&self) -> BridgeSubscriptionBasisKind {
        match self {
            Self::Snapshot { .. } => BridgeSubscriptionBasisKind::Snapshot,
            Self::BranchHead { .. } => BridgeSubscriptionBasisKind::BranchHead,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSubscriptionBasisBinding {
    basis_identity: BridgeSubscriptionBasisIdentity,
    basis_kind: BridgeSubscriptionBasisKind,
    branch_identity: Option<TruthBranchIdentity>,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: TruthSnapshotIdentity,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl ValidatedSubscriptionBasisBinding {
    pub(crate) fn bind(
        runtime: &RuntimeBridge,
        declaration: &BridgeSubscriptionDeclaration,
        request: &BridgeSubscriptionBasisRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeSubscriptionBasisResolutionFailure> {
        let (basis_kind, branch_identity, commit_identity, snapshot_identity) = match request {
            BridgeSubscriptionBasisRequest::Snapshot { snapshot_identity } => {
                bind_snapshot_basis(runtime, snapshot_identity, execution)?;
                (
                    BridgeSubscriptionBasisKind::Snapshot,
                    None,
                    None,
                    snapshot_identity.clone(),
                )
            }
            BridgeSubscriptionBasisRequest::BranchHead { branch_identity } => {
                let source = runtime.truth_branch_head_source.as_ref().ok_or_else(|| {
                    BridgeTypedError::new(
                        BridgeSubscriptionBasisResolutionFailureKind::MissingBranchHeadSource,
                        "Bridge runtime cannot bind branch-head subscription basis because no branch-head source is configured.",
                    )
                })?;
                let patch = source.load_branch_head_patch(branch_identity, execution).map_err(|error| {
                    BridgeTypedError::new(
                        BridgeSubscriptionBasisResolutionFailureKind::BranchHeadResolutionFailure,
                        format!(
                            "Bridge runtime failed to resolve branch head `{}` for subscription `{}`: {error}",
                            branch_identity.as_str(),
                            declaration.declaration_identity().as_str()
                        ),
                    )
                })?;
                if patch.branch_identity() != branch_identity {
                    return Err(BridgeTypedError::new(
                        BridgeSubscriptionBasisResolutionFailureKind::BranchHeadMismatch,
                        format!(
                            "Bridge runtime resolved branch head request `{}` for subscription `{}` onto branch `{}`.",
                            branch_identity.as_str(),
                            declaration.declaration_identity().as_str(),
                            patch.branch_identity().as_str()
                        ),
                    ));
                }
                bind_snapshot_basis(runtime, patch.snapshot_identity(), execution)?;
                (
                    BridgeSubscriptionBasisKind::BranchHead,
                    Some(branch_identity.clone()),
                    Some(patch.commit_identity().clone()),
                    patch.snapshot_identity().clone(),
                )
            }
        };

        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-basis-binding|declaration={}|basis-kind={}|branch={}|commit={}|snapshot={}",
            declaration.declaration_identity().as_str(),
            basis_kind.as_str(),
            branch_identity
                .as_ref()
                .map(TruthBranchIdentity::as_str)
                .unwrap_or("-"),
            commit_identity
                .as_ref()
                .map(TruthCommitIdentity::as_str)
                .unwrap_or("-"),
            snapshot_identity.as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            basis_identity: BridgeSubscriptionBasisIdentity::admit_bridge_owned(format!(
                "bridge-subscription-basis-id:sha256:{digest:x}"
            )),
            basis_kind,
            branch_identity,
            commit_identity,
            snapshot_identity,
            counters: BridgeSubscriptionCounters::from_basis_binding(),
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-basis-binding:sha256:{digest:x}"
            )),
        })
    }

    pub fn basis_identity(&self) -> &BridgeSubscriptionBasisIdentity {
        &self.basis_identity
    }

    pub fn basis_kind(&self) -> BridgeSubscriptionBasisKind {
        self.basis_kind
    }

    pub fn branch_identity(&self) -> Option<&TruthBranchIdentity> {
        self.branch_identity.as_ref()
    }

    pub fn commit_identity(&self) -> Option<&TruthCommitIdentity> {
        self.commit_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

fn bind_snapshot_basis(
    runtime: &RuntimeBridge,
    snapshot_identity: &TruthSnapshotIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), BridgeSubscriptionBasisResolutionFailure> {
    open_planned_snapshot(runtime, snapshot_identity, execution)
        .map(|_| ())
        .map_err(|error| match error.kind() {
            BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(_) => BridgeTypedError::new(
                BridgeSubscriptionBasisResolutionFailureKind::SnapshotAcquisitionFailure,
                format!(
                    "Bridge runtime failed to acquire snapshot `{}` for subscription basis binding: {}",
                    snapshot_identity.as_str(),
                    error
                ),
            ),
            BridgeDeliveryErrorKind::SnapshotIdentityMismatch => BridgeTypedError::new(
                BridgeSubscriptionBasisResolutionFailureKind::SnapshotIdentityMismatch,
                format!(
                    "Bridge runtime bound the wrong snapshot while proving subscription basis `{}`: {}",
                    snapshot_identity.as_str(),
                    error
                ),
            ),
            _ => BridgeTypedError::new(
                BridgeSubscriptionBasisResolutionFailureKind::SnapshotAcquisitionFailure,
                format!(
                    "Bridge runtime failed to prove subscription snapshot basis `{}`: {}",
                    snapshot_identity.as_str(),
                    error
                ),
            ),
        })
}

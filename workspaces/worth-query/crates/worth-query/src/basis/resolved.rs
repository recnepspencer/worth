use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceScope, WorthQueryEvidenceTag};
use crate::identity::BasisDigest;

use super::{BasisResolutionMode, ExecutionBasisIntent, ResolvedSnapshotIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedBasisProof {
    identity: WorthQueryEvidenceIdentity,
    digest: BasisDigest,
    resolution_mode: BasisResolutionMode,
    reporting_label: Option<String>,
}

impl ResolvedBasisProof {
    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn digest(&self) -> &BasisDigest {
        &self.digest
    }

    pub fn resolution_mode(&self) -> &BasisResolutionMode {
        &self.resolution_mode
    }

    pub fn reporting_label(&self) -> Option<&str> {
        self.reporting_label.as_deref()
    }

    pub(crate) fn new(
        identity: WorthQueryEvidenceIdentity,
        resolution_mode: BasisResolutionMode,
    ) -> Self {
        let digest = BasisDigest::from_evidence_identity(&identity);
        Self {
            identity,
            digest,
            resolution_mode,
            reporting_label: None,
        }
    }

    pub(crate) fn from_installed_operation(
        operation_identity: &str,
        basis_capability_identity: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::InstalledDomainExecution)
                .field_value(
                    WorthQueryEvidenceTag::new("installed_operation_identity"),
                    operation_identity,
                )
                .field_value(
                    WorthQueryEvidenceTag::new("basis_capability_identity"),
                    basis_capability_identity,
                )
                .seal()?;
        Ok(Self::new(identity, BasisResolutionMode::RuntimeDirect))
    }

    #[cfg(test)]
    pub(crate) fn from_digest_for_test(digest: BasisDigest) -> Self {
        Self {
            identity: digest.evidence_identity(),
            digest,
            resolution_mode: BasisResolutionMode::RuntimeDirect,
            reporting_label: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_identity_and_digest_for_test(
        identity: WorthQueryEvidenceIdentity,
        digest: BasisDigest,
    ) -> Self {
        Self {
            identity,
            digest,
            resolution_mode: BasisResolutionMode::RuntimeDirect,
            reporting_label: None,
        }
    }

    pub(crate) fn with_reporting_label(mut self, label: impl Into<String>) -> Self {
        self.reporting_label = Some(label.into());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedSnapshotBasis {
    intent: ExecutionBasisIntent,
    identity: ResolvedSnapshotIdentity,
    resolution_mode: BasisResolutionMode,
    proof: ResolvedBasisProof,
}

impl ResolvedSnapshotBasis {
    pub fn intent(&self) -> &ExecutionBasisIntent {
        &self.intent
    }

    pub fn identity(&self) -> &ResolvedSnapshotIdentity {
        &self.identity
    }

    pub fn resolution_mode(&self) -> &BasisResolutionMode {
        &self.resolution_mode
    }

    pub fn proof(&self) -> &ResolvedBasisProof {
        &self.proof
    }

    pub(crate) fn new(
        intent: ExecutionBasisIntent,
        identity: ResolvedSnapshotIdentity,
        resolution_mode: BasisResolutionMode,
    ) -> Self {
        let proof = ResolvedBasisProof::new(identity.evidence_identity(), resolution_mode.clone());
        Self {
            intent,
            identity,
            resolution_mode,
            proof,
        }
    }

    pub(crate) fn new_with_reporting_label(
        intent: ExecutionBasisIntent,
        identity: ResolvedSnapshotIdentity,
        resolution_mode: BasisResolutionMode,
        reporting_label: impl Into<String>,
    ) -> Self {
        let proof = ResolvedBasisProof::new(identity.evidence_identity(), resolution_mode.clone())
            .with_reporting_label(reporting_label);
        Self {
            intent,
            identity,
            resolution_mode,
            proof,
        }
    }

    #[cfg(test)]
    pub(crate) fn replace_proof_for_test(mut self, proof: ResolvedBasisProof) -> Self {
        self.proof = proof;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotResolutionReport {
    basis_identity: WorthQueryEvidenceIdentity,
    basis_digest: BasisDigest,
    resolution_mode: BasisResolutionMode,
    snapshot_basis_resolution_count: usize,
}

impl SnapshotResolutionReport {
    /// Returns whether this report was emitted for the exact resolved basis.
    ///
    /// Consumers should ask Query to certify this relationship instead of
    /// reconstructing it from rendered basis digests.
    pub fn certifies(&self, basis: &ResolvedSnapshotBasis) -> bool {
        self.basis_identity == *basis.proof().identity()
            && self.basis_digest == *basis.proof().digest()
            && self.resolution_mode == *basis.resolution_mode()
    }

    pub fn basis_digest(&self) -> &BasisDigest {
        &self.basis_digest
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn resolution_mode(&self) -> &BasisResolutionMode {
        &self.resolution_mode
    }

    pub fn snapshot_basis_resolution_count(&self) -> usize {
        self.snapshot_basis_resolution_count
    }

    pub(crate) fn from_resolved_basis(basis: &ResolvedSnapshotBasis) -> Self {
        Self {
            basis_identity: basis.proof().identity().clone(),
            basis_digest: basis.proof().digest().clone(),
            resolution_mode: basis.resolution_mode().clone(),
            snapshot_basis_resolution_count: 1,
        }
    }
}

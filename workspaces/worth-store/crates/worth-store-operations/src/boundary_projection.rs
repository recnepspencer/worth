use sha2::{Digest, Sha256};
use worth_foundational::{
    boundary_evidence, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceReceiptBoundary,
    FoundationalBoundaryEvidenceSourceBasis,
};
use worth_store_aspect_native::StoreExecutedBoundaryReceiptEvidence;
use worth_store_authority::StoreCurrentAuthorityWitness;

use crate::{ExecutedRepair, ExecutedRepairOwnerReceipt};

/// A support/audit projection derived from a closed Store repair execution.
///
/// The Store authorization and concrete owner-receipt identity remain beside
/// the Foundational evidence so the weaker projection can never become an
/// orchestration or readmission input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedRepairBoundaryProjection {
    evidence: StoreExecutedBoundaryReceiptEvidence,
    authorization_identity: [u8; 32],
    plan_fingerprint: [u8; 32],
    owner_receipt_fingerprint: [u8; 32],
    owner_receipt_count: u64,
}

impl ExecutedRepairBoundaryProjection {
    pub const fn evidence(&self) -> &StoreExecutedBoundaryReceiptEvidence {
        &self.evidence
    }
    pub const fn authorization_identity(&self) -> [u8; 32] {
        self.authorization_identity
    }
    pub const fn plan_fingerprint(&self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn owner_receipt_fingerprint(&self) -> [u8; 32] {
        self.owner_receipt_fingerprint
    }
    pub const fn owner_receipt_count(&self) -> u64 {
        self.owner_receipt_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairBoundaryProjectionDenial {
    StaleAuthority,
    FoundationalProvenance,
    ReceiptCountOverflow,
}

impl ExecutedRepair {
    pub fn project_execution_boundary(
        &self,
        current: &StoreCurrentAuthorityWitness,
    ) -> Result<ExecutedRepairBoundaryProjection, RepairBoundaryProjectionDenial> {
        if self.authority_identity() != current.authority_identity() {
            return Err(RepairBoundaryProjectionDenial::StaleAuthority);
        }
        let owner_receipt_count = self
            .owner_receipts()
            .receipts()
            .len()
            .try_into()
            .map_err(|_| RepairBoundaryProjectionDenial::ReceiptCountOverflow)?;
        let owner_receipt_fingerprint = owner_receipt_fingerprint(self);
        let plan_fingerprint = self.authorization().plan_fingerprint();
        let projection_identity = projection_identity(plan_fingerprint, owner_receipt_fingerprint);
        let locator = BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(u64::from_be_bytes(
                projection_identity[..8]
                    .try_into()
                    .expect("fixed digest prefix"),
            )),
            BoundaryArtifactField::Proofs,
        );
        let provenance = boundary_evidence()
            .provenance()
            .current(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                locator,
            ))
            .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained)
            .into_result()
            .map_err(|_| RepairBoundaryProjectionDenial::FoundationalProvenance)?;
        let receipt = boundary_evidence()
            .receipt()
            .execution(FoundationalBoundaryEvidenceReceiptBoundary::boundary_artifact(locator))
            .with_provenance(provenance);
        Ok(ExecutedRepairBoundaryProjection {
            evidence: StoreExecutedBoundaryReceiptEvidence::new(
                receipt,
                current.physical_witness(),
            ),
            authorization_identity: self.authorization().authorization_identity(),
            plan_fingerprint,
            owner_receipt_fingerprint,
            owner_receipt_count,
        })
    }
}

fn owner_receipt_fingerprint(executed: &ExecutedRepair) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-repair-owner-receipt-dag-v1");
    for (node, receipt) in executed.owner_receipts().receipts() {
        digest.update(node.fingerprint());
        match receipt {
            ExecutedRepairOwnerReceipt::Integrity(value) => {
                digest.update([2]);
                digest.update(value.plan_fingerprint());
                digest.update(value.classified_regions().to_be_bytes());
            }
        }
    }
    digest.finalize().into()
}

fn projection_identity(plan: [u8; 32], owners: [u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-executed-repair-boundary-projection-v1");
    digest.update(plan);
    digest.update(owners);
    digest.finalize().into()
}

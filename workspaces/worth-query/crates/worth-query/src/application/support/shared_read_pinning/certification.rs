use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::closure::{
    WorthQuerySharedReadPinningBoundaryClosure, WorthQuerySharedReadPinningBoundaryEvidence,
};
use super::evidence::{
    WorthQuerySharedReadPinningCounterEvidence, WorthQuerySharedReadPinningHostileMatrixEvidence,
    WorthQuerySharedReadPinningInventoryEvidence, WorthQuerySharedReadPortabilityEvidence,
    WorthQuerySharedReadStaleBasisDenialEvidence,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPinningCertification {
    inventory: WorthQuerySharedReadPinningInventoryEvidence,
    hostile_matrix: WorthQuerySharedReadPinningHostileMatrixEvidence,
    portability: WorthQuerySharedReadPortabilityEvidence,
    stale_basis_denial: WorthQuerySharedReadStaleBasisDenialEvidence,
    counters: WorthQuerySharedReadPinningCounterEvidence,
    closure: WorthQuerySharedReadPinningBoundaryClosure,
    artifact_digest: String,
    failure_digest: String,
}

impl WorthQuerySharedReadPinningCertification {
    pub fn from_evidence(
        inventory: WorthQuerySharedReadPinningInventoryEvidence,
        hostile_matrix: WorthQuerySharedReadPinningHostileMatrixEvidence,
        portability: WorthQuerySharedReadPortabilityEvidence,
        stale_basis_denial: WorthQuerySharedReadStaleBasisDenialEvidence,
        counters: WorthQuerySharedReadPinningCounterEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        let closure_evidence = WorthQuerySharedReadPinningBoundaryEvidence::new(
            inventory.total_failure_count(),
            counters.residue_count(),
            hostile_matrix.certified(),
            portability.proven_by_scoped_thread(),
            stale_basis_denial.proven_by_typed_denial(),
        );
        let closure =
            WorthQuerySharedReadPinningBoundaryClosure::derive_from_evidence(&closure_evidence)?;
        let failure_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_usize(
                    WorthQueryEvidenceTag::new("inventory_failure_count"),
                    inventory.total_failure_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("counter_residue_count"),
                    counters.residue_count(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("hostile_matrix_certified"),
                    hostile_matrix.certified(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("portability_proven"),
                    portability.proven_by_scoped_thread(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("stale_basis_denial_proven"),
                    stale_basis_denial.proven_by_typed_denial(),
                )
                .seal()?
                .as_str()
                .to_string();
        let artifact_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("shared_read_pinning_inventory_digest"),
            inventory.inventory_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("hostile_matrix_digest"),
            hostile_matrix.matrix_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("send_sync_proof_digest"),
            portability.scoped_thread_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("stale_basis_denial_digest"),
            stale_basis_denial.typed_denial_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("counter_digest"),
            counters.counter_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("closure_digest"),
            closure.closure_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("failure_digest"),
            &failure_digest,
        )
        .seal()?
        .as_str()
        .to_string();
        Ok(Self {
            inventory,
            hostile_matrix,
            portability,
            stale_basis_denial,
            counters,
            closure,
            artifact_digest,
            failure_digest,
        })
    }

    pub fn support_gate_required() -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::from_evidence(
            WorthQuerySharedReadPinningInventoryEvidence::new(
                0,
                0,
                "support-matrix-requires-runtime-pinning-certification",
            ),
            WorthQuerySharedReadPinningHostileMatrixEvidence::new(false, ""),
            WorthQuerySharedReadPortabilityEvidence::proven(""),
            WorthQuerySharedReadStaleBasisDenialEvidence::proven(""),
            WorthQuerySharedReadPinningCounterEvidence::new(0, 0, 0, 0, 0),
        )?)
    }

    pub fn closure(&self) -> &WorthQuerySharedReadPinningBoundaryClosure {
        &self.closure
    }
    #[cfg(test)]
    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }
    #[cfg(test)]
    pub fn failure_digest(&self) -> &str {
        &self.failure_digest
    }
}

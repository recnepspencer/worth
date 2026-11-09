use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySharedReadPinningBoundaryPosture {
    Open,
    Partial,
    Closed,
}

impl WorthQuerySharedReadPinningBoundaryPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Partial => "partial",
            Self::Closed => "closed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPinningBoundaryEvidence {
    inventory_failure_count: usize,
    counter_residue_count: usize,
    hostile_matrix_certified: bool,
    send_sync_proven: bool,
    stale_basis_denial_proven: bool,
}

impl WorthQuerySharedReadPinningBoundaryEvidence {
    pub(crate) fn new(
        inventory_failure_count: usize,
        counter_residue_count: usize,
        hostile_matrix_certified: bool,
        send_sync_proven: bool,
        stale_basis_denial_proven: bool,
    ) -> Self {
        Self {
            inventory_failure_count,
            counter_residue_count,
            hostile_matrix_certified,
            send_sync_proven,
            stale_basis_denial_proven,
        }
    }

    pub fn inventory_failure_count(&self) -> usize {
        self.inventory_failure_count
    }

    pub fn counter_residue_count(&self) -> usize {
        self.counter_residue_count
    }

    pub fn hostile_matrix_certified(&self) -> bool {
        self.hostile_matrix_certified
    }

    pub fn send_sync_proven(&self) -> bool {
        self.send_sync_proven
    }

    pub fn stale_basis_denial_proven(&self) -> bool {
        self.stale_basis_denial_proven
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedReadPinningBoundaryClosure {
    posture: WorthQuerySharedReadPinningBoundaryPosture,
    inventory_failure_count: usize,
    counter_residue_count: usize,
    hostile_matrix_green: bool,
    send_sync_proven: bool,
    stale_basis_denial_proven: bool,
    closure_identity: WorthQueryEvidenceIdentity,
}

impl WorthQuerySharedReadPinningBoundaryClosure {
    pub(crate) fn derive_from_evidence(
        evidence: &WorthQuerySharedReadPinningBoundaryEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        let posture = classify_shared_read_pinning_boundary_posture(
            evidence.inventory_failure_count(),
            evidence.counter_residue_count(),
            evidence.hostile_matrix_certified(),
            evidence.send_sync_proven(),
            evidence.stale_basis_denial_proven(),
        );
        let closure_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_shape(
                    WorthQueryEvidenceTag::new("shared_read_pinning_boundary_posture"),
                    posture.as_str(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("inventory_failure_count"),
                    evidence.inventory_failure_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("counter_residue_count"),
                    evidence.counter_residue_count(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("hostile_matrix_green"),
                    evidence.hostile_matrix_certified(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("send_sync_proven"),
                    evidence.send_sync_proven(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("stale_basis_denial_proven"),
                    evidence.stale_basis_denial_proven(),
                )
                .seal()?;
        Ok(Self {
            posture,
            inventory_failure_count: evidence.inventory_failure_count(),
            counter_residue_count: evidence.counter_residue_count(),
            hostile_matrix_green: evidence.hostile_matrix_certified(),
            send_sync_proven: evidence.send_sync_proven(),
            stale_basis_denial_proven: evidence.stale_basis_denial_proven(),
            closure_identity,
        })
    }

    pub fn posture(&self) -> WorthQuerySharedReadPinningBoundaryPosture {
        self.posture
    }

    pub fn closure_digest(&self) -> &str {
        self.closure_identity.as_str()
    }
    #[cfg(test)]
    pub fn inventory_failure_count(&self) -> usize {
        self.inventory_failure_count
    }
    #[cfg(test)]
    pub fn counter_residue_count(&self) -> usize {
        self.counter_residue_count
    }
    #[cfg(test)]
    pub fn hostile_matrix_green(&self) -> bool {
        self.hostile_matrix_green
    }
    #[cfg(test)]
    pub fn send_sync_proven(&self) -> bool {
        self.send_sync_proven
    }
    #[cfg(test)]
    pub fn stale_basis_denial_proven(&self) -> bool {
        self.stale_basis_denial_proven
    }
}

fn classify_shared_read_pinning_boundary_posture(
    inventory_failure_count: usize,
    counter_residue_count: usize,
    hostile_matrix_green: bool,
    send_sync_proven: bool,
    stale_basis_denial_proven: bool,
) -> WorthQuerySharedReadPinningBoundaryPosture {
    if inventory_failure_count == 0
        && counter_residue_count == 0
        && hostile_matrix_green
        && send_sync_proven
        && stale_basis_denial_proven
    {
        return WorthQuerySharedReadPinningBoundaryPosture::Closed;
    }
    if inventory_failure_count > 0 && counter_residue_count > 0 {
        return WorthQuerySharedReadPinningBoundaryPosture::Open;
    }
    WorthQuerySharedReadPinningBoundaryPosture::Partial
}

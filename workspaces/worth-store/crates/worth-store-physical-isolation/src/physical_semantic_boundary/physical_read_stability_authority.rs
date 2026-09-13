use worth_proof::{AuthorityMarker, AuthorityWitness};

use crate::{CompactionCutoverStabilityProof, PhysicalIsolationRootEpochBasis};

#[derive(Debug, Clone)]
pub struct PhysicalReadStabilityAuthority {
    root_epoch_basis: PhysicalIsolationRootEpochBasis,
}

#[derive(Debug, Clone)]
pub struct PhysicalReadStabilityCorrelationBasis {
    root_epoch_basis: PhysicalIsolationRootEpochBasis,
}

impl AuthorityMarker for PhysicalReadStabilityAuthority {}

#[cfg(any(test, feature = "certification-authority"))]
pub fn physical_read_stability_authority_for_certification_test(
    root_seed: u64,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
) -> PhysicalReadStabilityAuthority {
    let basis = crate::CurrentPhysicalRootBasis::new(
        crate::epoch::root_epoch_from_entry_seed(root_seed),
        crate::epoch::manifest_epoch_from_entry_seed(root_seed),
        store_authority_identity,
    );
    let root = crate::CurrentPhysicalRoot::from_physical_isolation_entry(
        basis,
        crate::PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .expect("certification root ordering should admit");
    PhysicalReadStabilityAuthority::from_current_root(root)
}

pub fn admit_post_compaction_read_stability_authority(
    proof: &CompactionCutoverStabilityProof,
) -> Result<PhysicalReadStabilityAuthority, core::convert::Infallible> {
    Ok(PhysicalReadStabilityAuthority::from_current_root(
        proof.post_cutover_root(),
    ))
}

pub fn admit_post_publication_read_stability_authority(
    receipt: &crate::PhysicalPublicationReceipt,
) -> Result<PhysicalReadStabilityAuthority, core::convert::Infallible> {
    Ok(PhysicalReadStabilityAuthority::from_current_root(
        receipt.new_root(),
    ))
}

impl PhysicalReadStabilityAuthority {
    fn from_current_root(root: crate::CurrentPhysicalRoot) -> Self {
        Self {
            root_epoch_basis: PhysicalIsolationRootEpochBasis::from_current_root(root),
        }
    }

    pub const fn root_epoch_basis(&self) -> PhysicalIsolationRootEpochBasis {
        self.root_epoch_basis
    }

    pub fn correlation_basis(&self) -> PhysicalReadStabilityCorrelationBasis {
        PhysicalReadStabilityCorrelationBasis {
            root_epoch_basis: self.root_epoch_basis,
        }
    }

    pub fn authority_witness(&self) -> AuthorityWitness<PhysicalReadStabilityAuthority> {
        AuthorityWitness::from_authority_marker(self.clone())
    }

    pub const fn is_store_physical_stability_authority(&self) -> bool {
        true
    }
}

impl PhysicalReadStabilityCorrelationBasis {
    pub const fn root_epoch_basis(&self) -> PhysicalIsolationRootEpochBasis {
        self.root_epoch_basis
    }
}

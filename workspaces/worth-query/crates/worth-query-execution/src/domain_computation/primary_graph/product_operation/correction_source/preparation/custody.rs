//! Home preparations retain identity and DATA; only Memory keeps live ancestry.
use super::super::occurrence::CorrectionOccurrence;
use super::*;
use crate::domain_computation::primary_graph::retained_correction::RetainedCorrectionMaterial;
use worth_foundational::facade::CorrectionMaterialPinIdentity;
use worth_runtime_world::facade::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

pub(super) enum PreparedCorrectionCustody {
    Memory {
        _ancestry: ProductBranchAncestorSelection,
        _alternatives: Option<(
            ProductBranchAncestorSelection,
            ProductBranchAncestorSelection,
        )>,
        material: CorrectionOccurrence,
    },
    Home {
        pin: CorrectionMaterialPinIdentity,
        fence: worth_proof::HomeIncarnationFence,
        world: CompositeCommitIdentity,
        commit: RelationalCommitReceipt,
        _material: RetainedCorrectionMaterial,
    },
}
impl PreparedCorrectionCustody {
    pub(super) fn commit(&self) -> &RelationalCommitReceipt {
        match self {
            Self::Memory { material, .. } => material.commit_reference(),
            Self::Home { commit, .. } => commit,
        }
    }
    pub(super) fn world(&self) -> &CompositeCommitIdentity {
        match self {
            Self::Memory { material, .. } => material
                .memory()
                .expect("Memory custody keeps actual Memory evidence")
                .committed_product_publication()
                .composite_commit(),
            Self::Home { world, .. } => world,
        }
    }
    pub(super) fn pin(&self) -> Option<CorrectionMaterialPinIdentity> {
        match self {
            Self::Memory { .. } => None,
            Self::Home { pin, .. } => Some(*pin),
        }
    }
    pub(super) fn admit_receiver(
        &self,
        owner: RuntimeWorldOwnerIdentity,
    ) -> Result<(), worth_proof::HomeIncarnationDenial> {
        use worth_proof::HomeIncarnationDenial as D;
        match self {
            Self::Memory { .. } => {
                if owner.durable().is_none() {
                    Ok(())
                } else {
                    Err(D::ForeignHome)
                }
            }
            Self::Home { pin, fence, .. } => {
                if owner.durable().map(|owner| owner.home()) != Some(pin.home()) {
                    return Err(D::ForeignHome);
                }
                fence.admit(owner.home_incarnation_fence().ok_or(D::ForeignHome)?)
            }
        }
    }
}

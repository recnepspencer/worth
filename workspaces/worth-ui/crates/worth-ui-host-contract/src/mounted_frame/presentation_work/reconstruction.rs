use super::{
    UiMountedLogicalDamage, UiMountedPaintCommand, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedPresentationAffinity,
    UiMountedPresentationAuxiliaryState,
};

#[derive(Debug, PartialEq)]
pub struct UiMountedPresentationReconstruction {
    pub(super) affinity: UiMountedPresentationAffinity,
    pub(super) projection: crate::UiMountedProjectionView,
    pub(super) auxiliary: UiMountedPresentationAuxiliaryState,
    pub(super) commands: Box<[UiMountedPaintCommand]>,
    pub(super) sample_overrides: Box<[super::UiMountedPresentationSampleChange]>,
    pub(super) order: Box<[UiMountedPaintOrderIdentity]>,
    pub(super) order_integrity: UiMountedPaintOrderIntegrity,
    pub(super) damage: Box<[UiMountedLogicalDamage]>,
    pub(super) production_cost: crate::UiMountedPresentationProductionCost,
}

#[doc(hidden)]
pub struct UiMountedPresentationReconstructionInput {
    pub predecessor: crate::UiMountedFrameIdentity,
    pub successor: crate::UiMountedFrameIdentity,
    pub surface: crate::UiSemanticSurfaceIdentity,
    pub binding: crate::UiSurfaceBindingGeneration,
    pub content: crate::UiMountedContentGeneration,
    pub baseline: crate::UiHostSurfaceBaselineIdentity,
    pub projection: crate::UiMountedProjectionView,
    pub commands: Vec<UiMountedPaintCommand>,
    pub sample_overrides: Vec<super::UiMountedPresentationSampleChange>,
    pub order: Vec<UiMountedPaintOrderIdentity>,
    pub order_integrity: UiMountedPaintOrderIntegrity,
    pub damage: Vec<UiMountedLogicalDamage>,
    pub production_cost: crate::UiMountedPresentationProductionCost,
}

impl UiMountedPresentationReconstruction {
    #[doc(hidden)]
    pub fn from_inert_mechanics(input: UiMountedPresentationReconstructionInput) -> Self {
        let receipt_affinity = input.projection.node_receipt_affinity();
        let auxiliary =
            UiMountedPresentationAuxiliaryState::from_runtime_mounting(&input.projection);
        let affinity = UiMountedPresentationAffinity::from_runtime(
            super::affinity::UiMountedPresentationAffinityInput {
                predecessor: Some(input.predecessor),
                successor: input.successor,
                surface: input.surface,
                binding: input.binding,
                content: input.content,
                baseline: input.baseline,
                receipt_affinity,
            },
        );
        Self {
            affinity,
            projection: input.projection,
            auxiliary,
            commands: input.commands.into_boxed_slice(),
            sample_overrides: input.sample_overrides.into_boxed_slice(),
            order: input.order.into_boxed_slice(),
            order_integrity: input.order_integrity,
            damage: input.damage.into_boxed_slice(),
            production_cost: input.production_cost,
        }
    }

    pub const fn affinity(&self) -> UiMountedPresentationAffinity {
        self.affinity
    }

    pub fn projection(&self) -> &crate::UiMountedProjectionView {
        &self.projection
    }

    pub fn auxiliary(&self) -> &UiMountedPresentationAuxiliaryState {
        &self.auxiliary
    }

    pub fn commands(&self) -> &[UiMountedPaintCommand] {
        &self.commands
    }

    pub fn sample_overrides(&self) -> &[super::UiMountedPresentationSampleChange] {
        &self.sample_overrides
    }

    pub fn order(&self) -> &[UiMountedPaintOrderIdentity] {
        &self.order
    }

    pub const fn order_integrity(&self) -> UiMountedPaintOrderIntegrity {
        self.order_integrity
    }

    pub fn damage(&self) -> &[UiMountedLogicalDamage] {
        &self.damage
    }

    pub const fn production_cost(&self) -> crate::UiMountedPresentationProductionCost {
        self.production_cost
    }
}

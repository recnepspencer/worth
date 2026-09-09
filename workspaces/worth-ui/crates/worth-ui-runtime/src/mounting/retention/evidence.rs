use worth_ui_host_contract::{
    UiHostPresentationEpoch, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSurfaceBindingGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiRetainedPresentationBinding {
    binding: UiSurfaceBindingGeneration,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    epoch: UiHostPresentationEpoch,
}

impl UiRetainedPresentationBinding {
    fn require_host(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<(), UiPresentedFrameBasisDenial> {
        if self.binding != presentation.binding()
            || self.host_surface != presentation.host_surface()
        {
            return Err(UiPresentedFrameBasisDenial::BindingNotPresented);
        }
        Ok(())
    }

    fn update_epoch(
        &mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<(), UiPresentedFrameBasisDenial> {
        self.require_host(presentation)?;
        self.epoch = presentation.epoch();
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct UiRetainedPresentedFrame {
    frame: UiMountedFrameIdentity,
    bindings: Box<[UiSurfaceBindingGeneration]>,
    presentation_bindings: Box<[UiRetainedPresentationBinding]>,
    presentation: Option<super::super::UiMountedPresentationReceipt>,
    receipts: super::super::UiMountedNodeReceiptBasis,
    visual_regions: super::super::UiMountedVisualRegionBasis,
    identity_trace_basis: super::super::UiMountedIdentityTraceBasis,
    structural_bytes: usize,
    mount_cost: super::super::UiMountCostReport,
}

pub(super) struct UiRetainedPresentedFrameInput {
    pub(super) frame: UiMountedFrameIdentity,
    pub(super) bindings: Box<[UiSurfaceBindingGeneration]>,
    pub(super) receipts: super::super::UiMountedNodeReceiptBasis,
    pub(super) mount_cost: super::super::UiMountCostReport,
    pub(super) visual_regions: super::super::UiMountedVisualRegionBasis,
    pub(super) identity_trace_basis: super::super::UiMountedIdentityTraceBasis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentedFrameBasisRelation {
    Current,
    Retained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentedFrameBasisDenial {
    Expired,
    Unknown,
    BindingNotPresented,
    PresentationEpochMismatch,
    PresentationTruthUnavailable,
    InstanceNotPresented,
    NodeReceiptMismatch,
}

impl UiRetainedPresentedFrame {
    pub(super) fn hit_index(&self) -> crate::mounting::presented_hit_index::UiPresentedHitIndex {
        self.visual_regions.presented_hits.clone()
    }

    pub(super) fn hit_presentations(
        &self,
    ) -> Vec<(
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        worth_ui_host_contract::UiHostObservationPresentationBasis,
    )> {
        self.current_presentations().collect()
    }

    pub(super) fn current_presentations(
        &self,
    ) -> impl Iterator<
        Item = (
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiHostObservationPresentationBasis,
        ),
    > + '_ {
        self.presentation
            .as_ref()
            .into_iter()
            .flat_map(|receipt| receipt.surfaces())
            .map(|surface| {
                let index = self
                    .presentation_bindings
                    .binary_search_by_key(&surface.binding(), |entry| entry.binding)
                    .expect("presented binding retains epoch");
                (
                    surface.semantic_surface(),
                    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                        surface.host_surface(),
                        self.frame,
                        surface.binding(),
                        self.presentation_bindings[index].epoch,
                    ),
                )
            })
    }

    pub(super) fn receipts(&self) -> &super::super::UiMountedNodeReceiptBasis {
        &self.receipts
    }

    pub(super) fn refresh_hit_motion(
        &mut self,
        sampler: &crate::mounting::presentation::motion_sampling::UiMountedMotionSampler,
        targets: &[crate::runtime::motion::UiMotionTargetIdentity],
    ) -> crate::mounting::hit_test_work::UiHitTestSpatialWork {
        let mut work = crate::mounting::hit_test_work::UiHitTestSpatialWork::default();
        if let Some(receipt) = &self.presentation {
            for surface in receipt.surfaces() {
                let targets = targets
                    .iter()
                    .copied()
                    .filter(|target| target.semantic_surface() == surface.semantic_surface())
                    .collect::<Vec<_>>();
                if targets.is_empty() {
                    continue;
                }
                let epoch = self
                    .presentation_bindings
                    .iter()
                    .find(|entry| entry.binding == surface.binding())
                    .map(|entry| entry.epoch)
                    .expect("presented binding has a current epoch");
                let presentation = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
                    surface.host_surface(),
                    self.frame,
                    surface.binding(),
                    epoch,
                );
                work.merge(self.visual_regions.presented_hits.apply_motion(
                    sampler,
                    presentation,
                    &targets,
                ));
            }
        }
        work
    }
    pub(super) fn prepare(input: UiRetainedPresentedFrameInput) -> Option<Self> {
        let mut bindings = input.bindings.into_vec();
        bindings.sort();
        bindings.dedup();
        let binding_bytes = bindings
            .len()
            .checked_mul(std::mem::size_of::<UiSurfaceBindingGeneration>())?;
        let presentation_surface_bytes = bindings.len().checked_mul(std::mem::size_of::<
            super::super::UiMountedSurfacePresentationReceipt,
        >())?;
        let presentation_binding_bytes = bindings
            .len()
            .checked_mul(std::mem::size_of::<UiRetainedPresentationBinding>())?;
        let visual_region_structural_bytes = input.visual_regions.retained_structural_bytes()?;
        let identity_trace_structural_bytes =
            input.identity_trace_basis.retained_structural_bytes()?;
        let structural_bytes = std::mem::size_of::<Self>()
            .checked_add(binding_bytes)?
            .checked_add(std::mem::size_of::<
                super::super::UiMountedPresentationReceipt,
            >())?
            .checked_add(presentation_surface_bytes)?
            .checked_add(presentation_binding_bytes)?
            .checked_add(input.receipts.retained_structural_bytes()?)?
            .checked_add(visual_region_structural_bytes)?
            .checked_add(input.visual_regions.motion_acceptance_reserved_bytes()?)?
            .checked_add(identity_trace_structural_bytes)?;
        Some(Self {
            frame: input.frame,
            bindings: bindings.into_boxed_slice(),
            presentation_bindings: Box::default(),
            presentation: None,
            receipts: input.receipts,
            visual_regions: input.visual_regions,
            identity_trace_basis: input.identity_trace_basis,
            structural_bytes,
            mount_cost: input.mount_cost,
        })
    }

    pub(crate) fn frame(&self) -> UiMountedFrameIdentity {
        self.frame
    }

    pub(crate) fn structural_bytes(&self) -> usize {
        self.structural_bytes
    }

    pub(crate) fn presented_binding_count(&self) -> usize {
        self.bindings.len()
    }

    pub(crate) fn mounted_instance_count(&self) -> usize {
        self.receipts.len()
    }

    pub(crate) fn mount_cost(&self) -> super::super::UiMountCostReport {
        self.mount_cost
    }

    pub(crate) fn visual_region_basis(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> super::super::UiMountedVisualRegionBasis {
        self.visual_regions
            .for_binding(binding, self.receipts.clone())
    }

    pub(crate) fn identity_trace_basis(&self) -> super::super::UiMountedIdentityTraceBasis {
        self.identity_trace_basis.clone()
    }

    pub(crate) fn projection_input(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<&worth_ui_query_binding::UiProjectionInputFactReference> {
        self.identity_trace_basis.projection_input(slot)
    }

    pub(crate) fn set_mount_cost(&mut self, mount_cost: super::super::UiMountCostReport) {
        self.mount_cost = mount_cost;
    }

    pub(crate) fn set_presentation_receipt(
        &mut self,
        presentation: super::super::UiMountedPresentationReceipt,
    ) {
        debug_assert_eq!(presentation.frame(), self.frame);
        let mut presentation_bindings = presentation
            .surfaces()
            .iter()
            .map(|surface| UiRetainedPresentationBinding {
                binding: surface.binding(),
                host_surface: surface.host_surface(),
                epoch: surface.epoch(),
            })
            .collect::<Vec<_>>();
        presentation_bindings.sort_by_key(|entry| entry.binding);
        self.presentation_bindings = presentation_bindings.into_boxed_slice();
        self.presentation = Some(presentation);
    }

    pub(crate) fn update_presentation_epoch(
        &mut self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<(), UiPresentedFrameBasisDenial> {
        if presentation.frame() != self.frame {
            return Err(UiPresentedFrameBasisDenial::Unknown);
        }
        let index = self
            .presentation_bindings
            .binary_search_by_key(&presentation.binding(), |entry| entry.binding)
            .map_err(|_| UiPresentedFrameBasisDenial::BindingNotPresented)?;
        self.presentation_bindings[index].update_epoch(presentation)
    }

    pub(crate) fn presentation_receipt(
        &self,
    ) -> Option<&super::super::UiMountedPresentationReceipt> {
        self.presentation.as_ref()
    }

    pub(crate) fn receipt_for_with_probes(
        &self,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> (Option<UiMountedNodeReceiptIdentity>, usize) {
        self.receipts.receipt_for_with_probes(mounted_instance)
    }

    pub(crate) fn classify(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        mounted_instance: Option<UiMountedInstanceIdentity>,
        node_receipt: Option<UiMountedNodeReceiptIdentity>,
    ) -> Result<(), UiPresentedFrameBasisDenial> {
        let binding = presentation.binding();
        if self.bindings.binary_search(&binding).is_err() {
            return Err(UiPresentedFrameBasisDenial::BindingNotPresented);
        }
        let retained_binding = self
            .presentation_bindings
            .binary_search_by_key(&binding, |entry| entry.binding)
            .ok()
            .map(|index| &self.presentation_bindings[index])
            .ok_or(UiPresentedFrameBasisDenial::BindingNotPresented)?;
        retained_binding.require_host(presentation)?;
        if retained_binding.epoch != presentation.epoch() {
            return Err(UiPresentedFrameBasisDenial::PresentationEpochMismatch);
        }
        match (mounted_instance, node_receipt) {
            (None, None) => Ok(()),
            (Some(instance), Some(receipt)) => {
                let expected = self
                    .receipts
                    .receipt_for(instance)
                    .ok_or(UiPresentedFrameBasisDenial::InstanceNotPresented)?;
                (expected == receipt)
                    .then_some(())
                    .ok_or(UiPresentedFrameBasisDenial::NodeReceiptMismatch)
            }
            _ => Err(UiPresentedFrameBasisDenial::InstanceNotPresented),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrong_host_cannot_update_a_retained_binding_epoch() {
        use worth_ui_host_contract::*;
        let mut retained = UiRetainedPresentationBinding {
            binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            host_surface: UiHostSurfaceIdentity::mint_unbound().unwrap(),
            epoch: UiHostPresentationEpoch::issued_by_host(1),
        };
        let before = retained;
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let wrong = UiHostObservationPresentationBasis::new(
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            frame,
            retained.binding,
            UiHostPresentationEpoch::issued_by_host(2),
        );
        assert_eq!(
            retained.update_epoch(wrong),
            Err(UiPresentedFrameBasisDenial::BindingNotPresented)
        );
        assert_eq!(retained, before);
        let right = UiHostObservationPresentationBasis::new(
            retained.host_surface,
            frame,
            retained.binding,
            wrong.epoch(),
        );
        retained.update_epoch(right).unwrap();
        assert_eq!(retained.epoch, right.epoch());
        assert_eq!(
            (retained.binding, retained.host_surface),
            (before.binding, before.host_surface)
        );
    }
}

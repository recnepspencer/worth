use super::{
    FrozenMosaicRegionCapabilities, MosaicRegionAcceptedRegistrationProof,
    MosaicRegionKindDescriptor,
};

/// Builder-owned mosaic region kind registry lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MosaicRegionRegistry {
    descriptors: Vec<MosaicRegionKindDescriptor>,
    seam_paint: Option<super::MosaicSeamPaintContract>,
}

impl MosaicRegionRegistry {
    pub(crate) const SEAM_REGISTRATION_IDENTITY: &'static str = "__mosaic_seam_paint_contract__";
    pub(crate) fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
            seam_paint: None,
        }
    }

    pub(crate) fn install_seam_paint(
        &mut self,
        contract: super::MosaicSeamPaintContract,
    ) -> Result<crate::capability::RegistrationCandidate, super::MosaicSeamPaintContractDenial>
    {
        if self.seam_paint.is_some() {
            return Err(super::MosaicSeamPaintContractDenial::DuplicateContract);
        }
        let candidate = contract.regions().iter().fold(
            crate::capability::RegistrationCandidate::new(
                crate::capability::MOSAIC_SEAM_PAINT_FAMILY_NAME,
                Self::SEAM_REGISTRATION_IDENTITY,
                crate::capability::CapabilitySupportKind::Admitted,
            ),
            |candidate, region| {
                candidate.with_dependency(crate::capability::RegistrationDependency::new(
                    crate::capability::MOSAIC_REGION_KIND_FAMILY_NAME,
                    crate::capability::MOSAIC_REGION_KIND_FAMILY_NAME,
                    region.as_str(),
                ))
            },
        );
        self.seam_paint = Some(contract);
        Ok(candidate)
    }

    pub(crate) fn seam_paint_matches_registered_regions(&self) -> bool {
        let Some(contract) = &self.seam_paint else {
            return true;
        };
        let mut registered = self
            .descriptors
            .iter()
            .map(|descriptor| descriptor.id().clone())
            .collect::<Vec<_>>();
        registered.sort();
        registered == contract.regions()
    }

    pub(crate) fn push(&mut self, descriptor: MosaicRegionKindDescriptor) {
        self.descriptors.push(descriptor);
    }

    pub(crate) fn freeze(
        self,
        accepted_regions: &MosaicRegionAcceptedRegistrationProof,
        accepted_seam_paint: &super::MosaicSeamPaintAcceptedRegistrationProof,
    ) -> FrozenMosaicRegionCapabilities {
        FrozenMosaicRegionCapabilities::from_accepted_descriptors(
            self.descriptors,
            accepted_regions,
            self.seam_paint
                .filter(|_| accepted_seam_paint.admits_contract()),
        )
    }
}

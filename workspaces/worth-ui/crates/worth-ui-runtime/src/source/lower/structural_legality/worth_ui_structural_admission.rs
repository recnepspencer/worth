use super::worth_ui_structural_legality_context::{
    PlacementResolution, SizingResolution, StateResolution, WorthUiStructuralLegalityContext,
};
use super::worth_ui_structural_semantics::{
    mount_state_slot_is_legal, placement_policy_matches_mount, region_state_slot_is_legal,
    sizing_contract_matches_region,
};
use crate::capability::{
    MosaicChildRule, MosaicRegionKindDescriptor, MosaicRegionRole, SurfaceDescriptor,
};
use crate::source::{
    WorthUiMosaicMountFacts, WorthUiMosaicRegionFacts, WorthUiMosaicStructureFacts,
    WorthUiStructuralLegalityDiagnostic, WorthUiStructuralLegalityDiagnosticCode,
};
use worth_ui_dsl::{WorthUiAuthoredMount, WorthUiAuthoredRegion, WorthUiAuthoredStructuralBody};

pub(super) fn admit_structure(
    module_id: &worth_ui_dsl::WorthUiSourceModuleId,
    authored_body: &WorthUiAuthoredStructuralBody,
    provenance: &worth_ui_dsl::WorthUiArtifactInputProvenance,
    context: &mut WorthUiStructuralLegalityContext<'_>,
) -> Result<WorthUiMosaicStructureFacts, Vec<WorthUiStructuralLegalityDiagnostic>> {
    let mut admission = StructuralAdmission {
        module_id,
        provenance,
        context,
    };
    let mut diagnostics = Vec::new();
    let mut root_regions = Vec::new();

    for (index, region) in authored_body.root_regions().iter().enumerate() {
        let locus = format!("root[{index}]");
        match admission.lower_region(region, &locus) {
            Ok(region) => root_regions.push(region),
            Err(mut region_diagnostics) => diagnostics.append(&mut region_diagnostics),
        }
    }

    if diagnostics.is_empty() {
        Ok(WorthUiMosaicStructureFacts::new(root_regions))
    } else {
        Err(diagnostics)
    }
}

struct StructuralAdmission<'input, 'snapshot> {
    module_id: &'input worth_ui_dsl::WorthUiSourceModuleId,
    provenance: &'input worth_ui_dsl::WorthUiArtifactInputProvenance,
    context: &'input mut WorthUiStructuralLegalityContext<'snapshot>,
}

impl StructuralAdmission<'_, '_> {
    fn lower_region(
        &mut self,
        region: &WorthUiAuthoredRegion,
        structural_locus: &str,
    ) -> Result<WorthUiMosaicRegionFacts, Vec<WorthUiStructuralLegalityDiagnostic>> {
        let (resolved_region, descriptor) = self
            .context
            .resolve_region(
                self.module_id,
                region.region_id_text(),
                structural_locus,
                self.provenance,
            )
            .map_err(|diagnostic| vec![diagnostic])?;
        let mut diagnostics = Vec::new();
        let sizing_contract =
            self.admit_region_sizing(region, &descriptor, structural_locus, &mut diagnostics)?;
        let state_slot =
            self.admit_region_state(region, &descriptor, structural_locus, &mut diagnostics)?;
        let child_regions = self.lower_child_regions(region, structural_locus, &mut diagnostics);
        let mounts = self.lower_region_mounts(
            region,
            descriptor.role(),
            structural_locus,
            &mut diagnostics,
        );
        self.validate_region_children(
            &descriptor,
            &child_regions,
            &mounts,
            structural_locus,
            &mut diagnostics,
        );

        if diagnostics.is_empty() {
            Ok(WorthUiMosaicRegionFacts::new(
                resolved_region,
                descriptor,
                sizing_contract,
                state_slot,
                child_regions,
                mounts,
            ))
        } else {
            Err(diagnostics)
        }
    }

    fn admit_region_sizing(
        &mut self,
        region: &WorthUiAuthoredRegion,
        descriptor: &MosaicRegionKindDescriptor,
        structural_locus: &str,
        diagnostics: &mut Vec<WorthUiStructuralLegalityDiagnostic>,
    ) -> Result<Option<SizingResolution>, Vec<WorthUiStructuralLegalityDiagnostic>> {
        let sizing_contract = region
            .sizing_contract_id_text()
            .map(|authored_text| {
                self.context.resolve_sizing(
                    self.module_id,
                    authored_text,
                    structural_locus,
                    self.provenance,
                )
            })
            .transpose()
            .map_err(|diagnostic| vec![diagnostic])?;
        if let Some((_, sizing_descriptor)) = sizing_contract.as_ref() {
            let matches = descriptor.sizing_behavior().is_some_and(|behavior| {
                sizing_contract_matches_region(behavior, sizing_descriptor.kind())
            });
            if !matches {
                diagnostics.push(self.diagnostic(
                    WorthUiStructuralLegalityDiagnosticCode::IllegalSizingContractForRegion,
                    sizing_descriptor.id().as_str(),
                    structural_locus,
                ));
            }
        }
        Ok(sizing_contract)
    }

    fn admit_region_state(
        &mut self,
        region: &WorthUiAuthoredRegion,
        descriptor: &MosaicRegionKindDescriptor,
        structural_locus: &str,
        diagnostics: &mut Vec<WorthUiStructuralLegalityDiagnostic>,
    ) -> Result<Option<StateResolution>, Vec<WorthUiStructuralLegalityDiagnostic>> {
        let state_slot = region
            .state_slot_id_text()
            .map(|authored_text| {
                self.context.resolve_state_slot(
                    self.module_id,
                    authored_text,
                    structural_locus,
                    self.provenance,
                )
            })
            .transpose()
            .map_err(|diagnostic| vec![diagnostic])?;
        if let (Some((_, state_descriptor)), Some(scroll_ownership)) =
            (state_slot.as_ref(), descriptor.scroll_ownership())
            && let Err(code) = region_state_slot_is_legal(
                descriptor.id(),
                descriptor.role(),
                scroll_ownership,
                state_descriptor,
            )
        {
            diagnostics.push(self.diagnostic(
                code,
                state_descriptor.id().as_str(),
                structural_locus,
            ));
        }
        Ok(state_slot)
    }

    fn lower_child_regions(
        &mut self,
        region: &WorthUiAuthoredRegion,
        structural_locus: &str,
        diagnostics: &mut Vec<WorthUiStructuralLegalityDiagnostic>,
    ) -> Vec<WorthUiMosaicRegionFacts> {
        let mut children = Vec::new();
        for (index, child) in region.child_regions().iter().enumerate() {
            match self.lower_region(child, &format!("{structural_locus}/region[{index}]")) {
                Ok(child) => children.push(child),
                Err(mut child_diagnostics) => diagnostics.append(&mut child_diagnostics),
            }
        }
        children
    }

    fn lower_region_mounts(
        &mut self,
        region: &WorthUiAuthoredRegion,
        region_role: &MosaicRegionRole,
        structural_locus: &str,
        diagnostics: &mut Vec<WorthUiStructuralLegalityDiagnostic>,
    ) -> Vec<WorthUiMosaicMountFacts> {
        let mut mounts = Vec::new();
        for mount in region.mounts() {
            match self.lower_mount(mount, region_role, structural_locus) {
                Ok(mount) => mounts.push(mount),
                Err(mut mount_diagnostics) => diagnostics.append(&mut mount_diagnostics),
            }
        }
        mounts
    }

    fn validate_region_children(
        &self,
        descriptor: &MosaicRegionKindDescriptor,
        child_regions: &[WorthUiMosaicRegionFacts],
        mounts: &[WorthUiMosaicMountFacts],
        structural_locus: &str,
        diagnostics: &mut Vec<WorthUiStructuralLegalityDiagnostic>,
    ) {
        let code = match descriptor.child_rule() {
            Some(MosaicChildRule::AcceptsSurfaces) if !child_regions.is_empty() => {
                Some(WorthUiStructuralLegalityDiagnosticCode::IllegalRegionChildMix)
            }
            Some(MosaicChildRule::AcceptsRegions | MosaicChildRule::AcceptsRegionStack)
                if !mounts.is_empty() =>
            {
                Some(WorthUiStructuralLegalityDiagnosticCode::IllegalSurfaceMountInRegion)
            }
            Some(MosaicChildRule::LeafOnly) if !mounts.is_empty() || !child_regions.is_empty() => {
                Some(WorthUiStructuralLegalityDiagnosticCode::IllegalLeafRegionChildren)
            }
            _ => None,
        };
        if let Some(code) = code {
            diagnostics.push(self.diagnostic(code, descriptor.id().as_str(), structural_locus));
        }
    }

    fn lower_mount(
        &mut self,
        mount: &WorthUiAuthoredMount,
        region_role: &MosaicRegionRole,
        structural_locus: &str,
    ) -> Result<WorthUiMosaicMountFacts, Vec<WorthUiStructuralLegalityDiagnostic>> {
        let mount_locus = format!("{structural_locus}/mount:{}", mount.surface_id_text());
        let (surface, descriptor) = self
            .context
            .resolve_surface(
                self.module_id,
                mount.surface_id_text(),
                &mount_locus,
                self.provenance,
            )
            .map_err(|diagnostic| vec![diagnostic])?;
        let placement_policy = self.resolve_mount_placement(mount, &mount_locus)?;
        let state_slot = self.resolve_mount_state(mount, &mount_locus)?;
        let diagnostics = self.validate_mount(
            &descriptor,
            region_role,
            placement_policy.as_ref(),
            state_slot.as_ref(),
            &mount_locus,
        );
        if diagnostics.is_empty() {
            Ok(WorthUiMosaicMountFacts::new(
                surface,
                descriptor,
                placement_policy,
                state_slot,
            ))
        } else {
            Err(diagnostics)
        }
    }

    fn resolve_mount_placement(
        &mut self,
        mount: &WorthUiAuthoredMount,
        mount_locus: &str,
    ) -> Result<Option<PlacementResolution>, Vec<WorthUiStructuralLegalityDiagnostic>> {
        mount
            .placement_policy_id_text()
            .map(|authored_text| {
                self.context.resolve_placement(
                    self.module_id,
                    authored_text,
                    mount_locus,
                    self.provenance,
                )
            })
            .transpose()
            .map_err(|diagnostic| vec![diagnostic])
    }

    fn resolve_mount_state(
        &mut self,
        mount: &WorthUiAuthoredMount,
        mount_locus: &str,
    ) -> Result<Option<StateResolution>, Vec<WorthUiStructuralLegalityDiagnostic>> {
        mount
            .state_slot_id_text()
            .map(|authored_text| {
                self.context.resolve_state_slot(
                    self.module_id,
                    authored_text,
                    mount_locus,
                    self.provenance,
                )
            })
            .transpose()
            .map_err(|diagnostic| vec![diagnostic])
    }

    fn validate_mount(
        &self,
        descriptor: &SurfaceDescriptor,
        region_role: &MosaicRegionRole,
        placement_policy: Option<&PlacementResolution>,
        state_slot: Option<&StateResolution>,
        mount_locus: &str,
    ) -> Vec<WorthUiStructuralLegalityDiagnostic> {
        let mut diagnostics = Vec::new();
        if let Some((_, policy_descriptor)) = placement_policy
            && !placement_policy_matches_mount(descriptor, region_role, policy_descriptor)
        {
            diagnostics.push(self.diagnostic(
                WorthUiStructuralLegalityDiagnosticCode::IllegalPlacementPolicyForMount,
                policy_descriptor.id().as_str(),
                mount_locus,
            ));
        }
        if let Some((_, state_descriptor)) = state_slot
            && let Err(code) =
                mount_state_slot_is_legal(descriptor.id(), descriptor, state_descriptor)
        {
            diagnostics.push(self.diagnostic(code, state_descriptor.id().as_str(), mount_locus));
        }
        diagnostics
    }

    fn diagnostic(
        &self,
        code: WorthUiStructuralLegalityDiagnosticCode,
        authored_text: &str,
        structural_locus: &str,
    ) -> WorthUiStructuralLegalityDiagnostic {
        WorthUiStructuralLegalityDiagnostic::new(
            code,
            self.module_id.clone(),
            authored_text,
            structural_locus,
            self.provenance.clone(),
        )
    }
}

use super::UiMountedAppearanceLoweringDenial;
use std::collections::BTreeMap;
use worth_ui_host_contract::{
    UiAppearanceLogicalLength, UiHostAppearanceProfileContract, UiMountedSurfaceBindingRequirement,
    UiSemanticSurfaceIdentity,
};

/// Frame-local physical qualification from admitted host capability and binding facts.
pub(crate) struct UiMountedAppearanceGeometryScope {
    fringe_by_surface: BTreeMap<
        UiSemanticSurfaceIdentity,
        Result<UiAppearanceLogicalLength, UiMountedAppearanceLoweringDenial>,
    >,
    motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
}

impl UiMountedAppearanceGeometryScope {
    pub(crate) fn new(
        bindings: &[UiMountedSurfaceBindingRequirement],
        profile: Option<&UiHostAppearanceProfileContract>,
    ) -> Self {
        let fringe_by_surface = bindings
            .iter()
            .map(|binding| {
                let fringe = profile
                    .ok_or(UiMountedAppearanceLoweringDenial::HostGeometryProfileUnavailable)
                    .and_then(|profile| {
                        profile
                            .geometry_qualification()
                            .row_for_scale(binding.device_scale_milli())
                            .map(|row| row.anti_alias_fringe_logical_subpixels())
                            .map_err(UiMountedAppearanceLoweringDenial::HostGeometryScale)
                    });
                (binding.semantic_surface(), fringe)
            })
            .collect();
        Self {
            fringe_by_surface,
            motion: Default::default(),
        }
    }

    pub(crate) fn with_motion(
        bindings: &[UiMountedSurfaceBindingRequirement],
        profile: Option<&UiHostAppearanceProfileContract>,
        motion: crate::mounting::presentation::UiAcceptedAppearanceMotion,
    ) -> Self {
        let mut scope = Self::new(bindings, profile);
        scope.motion = motion;
        scope
    }

    pub(crate) fn outline_fringe(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiAppearanceLogicalLength, UiMountedAppearanceLoweringDenial> {
        self.fringe_by_surface.get(&surface).cloned().unwrap_or(Err(
            UiMountedAppearanceLoweringDenial::HostGeometrySurfaceUnavailable,
        ))
    }

    pub(crate) fn includes_surface(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.fringe_by_surface.contains_key(&surface)
    }

    pub(crate) fn motion_opacity(
        &self,
        command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    ) -> Option<Option<u16>> {
        self.motion.opacity_for(command)
    }

    pub(crate) fn instance_motion_opacity(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        portal_surface: bool,
    ) -> Result<Option<Option<u16>>, UiMountedAppearanceLoweringDenial> {
        self.motion
            .opacity_for_instance(instance, portal_surface)
            .map_err(|_| UiMountedAppearanceLoweringDenial::AmbiguousMotionOpacity)
    }
}

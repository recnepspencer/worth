//! Raw mounted geometry dependencies retained with accepted appearance facts.

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedAppearanceGeometryInput {
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    plan_digest: u64,
    layout_allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    clip: super::UiMountedAppearanceClip,
    order: Option<u32>,
    surface_geometry: worth_ui_host_contract::UiSurfaceGeometry,
    portal_child_owner: Option<crate::capability::ComponentId>,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    surface_paint_posture: crate::mounting::UiMountedSurfacePaintPosture,
}

impl UiMountedAppearanceGeometryInput {
    pub(super) fn surface_geometry(&self) -> worth_ui_host_contract::UiSurfaceGeometry {
        self.surface_geometry.clone()
    }

    pub(in crate::mounting::projection) fn from_node(
        node: &super::super::frame_storage::UiMountedProjectionNodeRecord,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Self {
        Self {
            instance: node.receipt().mounted_instance(),
            plan_digest: node.receipt().plan_digest(),
            layout_allocation: node.receipt().allocation(),
            allocation: node
                .completed_appearance_geometry()
                .allocation()
                .into_shown(),
            clip: node.appearance_clip,
            order: node.surface_paint_order,
            surface_geometry: node.surface_geometry.clone(),
            portal_child_owner: node.portal_child_owner.clone(),
            binding,
            surface_paint_posture: node.completed_appearance_geometry().surface_paint_posture(),
        }
    }

    pub(super) fn surface_paint_posture(&self) -> crate::mounting::UiMountedSurfacePaintPosture {
        self.surface_paint_posture.clone()
    }
}

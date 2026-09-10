use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::appearance::text_foreground::UiNativeTextForegroundJoin;
use crate::native::presentation::appearance::{
    UiNativeAppearanceCommand, UiNativeAppearanceRetained, UiNativeAppearanceScale,
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedFrameConsumptionView, UiMountedOverlayOrderMechanic,
};

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn has_appearance(&self) -> bool {
        self.staged_appearance.is_some()
    }

    pub(in crate::native::presentation) fn appearance_surface_operation(
        &self,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        extent: [u32; 2],
    ) -> Result<Option<crate::native::presentation::UiNativeRasterOperation>, Denial> {
        self.appearance_surface_operation_for_identity(
            crate::native::presentation::appearance::UiNativeAppearanceCommandIdentity::Surface(
                node_receipt.mounted_instance(),
            ),
            extent,
        )
    }

    pub(in crate::native::presentation) fn appearance_portal_surface_operation(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        extent: [u32; 2],
    ) -> Result<Option<crate::native::presentation::UiNativeRasterOperation>, Denial> {
        self.appearance_surface_operation_for_identity(
            crate::native::presentation::appearance::UiNativeAppearanceCommandIdentity::PortalSurface(
                instance,
            ),
            extent,
        )
    }

    fn appearance_surface_operation_for_identity(
        &self,
        identity: crate::native::presentation::appearance::UiNativeAppearanceCommandIdentity,
        extent: [u32; 2],
    ) -> Result<Option<crate::native::presentation::UiNativeRasterOperation>, Denial> {
        let Some((_, appearance)) = &self.staged_appearance else {
            return Ok(None);
        };
        let Some(key) = appearance.key_for_identity(&identity) else {
            return Ok(None);
        };
        let mechanic = match appearance.command(key).ok_or(Denial::CommandMismatch)? {
            UiNativeAppearanceCommand::Surface(mechanic) => mechanic,
            UiNativeAppearanceCommand::PortalSurface(mechanic) => mechanic.surface(),
            _ => return Err(Denial::CommandMismatch),
        };
        crate::native::presentation::appearance::UiNativeSurfacePipeline::prepare(
            mechanic,
            appearance.scale(),
        )
        .and_then(|primitive| primitive.raster_operation(extent))
        .map(|operation| {
            operation.map(crate::native::presentation::UiNativeRasterOperation::Surface)
        })
        .map_err(|_| Denial::CommandMismatch)
    }

    pub(in crate::native::presentation) fn initialize_appearance(
        &mut self,
        view: &UiMountedFrameConsumptionView<'_>,
        atlas: &UiNativeTextAtlas,
        extent: [u32; 2],
    ) -> Result<(), Denial> {
        let work = view.appearance_work().ok_or(Denial::CommandMismatch)?;
        if self.staged_appearance.is_some()
            || work.frame() != self.frame
            || work.presentation() != view.attempt()
            || work.requirement() != view.requirement()
        {
            return Err(Denial::AffinityMismatch);
        }
        let scale = u16::try_from(work.requirement().device_scale_milli())
            .map_err(|_| Denial::AffinityMismatch)?;
        let scale =
            UiNativeAppearanceScale::qualified(scale).map_err(|_| Denial::AffinityMismatch)?;
        let mut retained = UiNativeAppearanceRetained::new(scale);
        let mut predecessor = None;
        let mut overlay: Option<UiMountedOverlayOrderMechanic> = None;

        for fragment in work.fragments() {
            if fragment.surface_binding() != work.requirement()
                || fragment.presentation_affinity().successor() != self.frame
                || fragment.presentation_affinity().surface() != self.surface
                || fragment.presentation_affinity().binding() != self.binding
                || fragment.presentation_affinity().content() != self.content
            {
                return Err(Denial::AffinityMismatch);
            }
            let successor = fragment.work().successor();
            if successor.frame() != self.frame || successor.semantic_surface() != self.surface {
                return Err(Denial::AffinityMismatch);
            }
            match &overlay {
                Some(current) if current != successor.overlay_order() => {
                    return Err(Denial::CommandMismatch)
                }
                None => overlay = Some(successor.overlay_order().clone()),
                Some(_) => {}
            }
            for mechanic in successor.mechanics() {
                let command = match mechanic {
                    UiMountedAppearanceMechanic::Surface(value) => {
                        UiNativeAppearanceCommand::Surface(value.clone())
                    }
                    UiMountedAppearanceMechanic::PortalSurface(value) => {
                        UiNativeAppearanceCommand::PortalSurface(value.clone())
                    }
                    UiMountedAppearanceMechanic::Outline(value) => {
                        UiNativeAppearanceCommand::Outline(value.clone())
                    }
                    UiMountedAppearanceMechanic::TextForeground(value) => {
                        let (foreground, _) =
                            UiNativeTextForegroundJoin::admit(fragment, view, value)
                                .and_then(|join| join.finalize(atlas, extent))
                                .map_err(|_| Denial::CommandMismatch)?;
                        self.validate_text_coverage(&foreground)?;
                        UiNativeAppearanceCommand::TextForeground(foreground)
                    }
                    UiMountedAppearanceMechanic::Backdrop(value) => {
                        UiNativeAppearanceCommand::Backdrop(value.clone())
                    }
                    UiMountedAppearanceMechanic::Pointer(value) => {
                        UiNativeAppearanceCommand::PointerAffordance(value.clone())
                    }
                };
                predecessor = Some(
                    retained
                        .insert(command, predecessor)
                        .map_err(|_| Denial::CommandMismatch)?,
                );
            }
        }
        let overlay = overlay.ok_or(Denial::CommandMismatch)?;
        retained
            .insert(
                UiNativeAppearanceCommand::OverlayOrder(overlay),
                predecessor,
            )
            .map_err(|_| Denial::CommandMismatch)?;
        retained.take_damage();
        self.staged_appearance = Some((work.requirement(), retained));
        Ok(())
    }
}

use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedPaintCommandIdentity};

use super::super::motion_evidence::UiCommandMotionAcceptance;
use super::UiMountedPresentationState;
use crate::mounting::projection::UiMountedAppearanceSurfaceSampleGeometry;

/// The painted surface of an appearance-only instance bound as a Motion
/// target, with the live acceptance slot its unchanged geometry keeps.
#[derive(Clone)]
pub(in crate::mounting::presentation) struct UiMountedAppearanceSurfaceSampleTarget {
    geometry: UiMountedAppearanceSurfaceSampleGeometry,
    motion: UiCommandMotionAcceptance,
}

impl UiMountedAppearanceSurfaceSampleTarget {
    pub(in crate::mounting::presentation::work_producer) const fn geometry(
        &self,
    ) -> UiMountedAppearanceSurfaceSampleGeometry {
        self.geometry
    }

    pub(in crate::mounting::presentation::work_producer) const fn motion(
        &self,
    ) -> &UiCommandMotionAcceptance {
        &self.motion
    }
}

impl UiMountedPresentationState {
    /// Rebinds the lowered appearance of each target on this surface: its raw
    /// opacity, and its surface as a sample target when it owns no paint command.
    pub(in crate::mounting::presentation) fn bind_appearance_sample_targets(
        &mut self,
        frame: &crate::mounting::UiPreparedMountedFrame,
        targets: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            UiMountedInstanceIdentity,
        )],
    ) {
        for &(surface, instance) in targets {
            if surface != self.requirement.semantic_surface() {
                continue;
            }
            if let Some(opacity) = frame.appearance_raw_opacity_for_instance(instance) {
                self.appearance_opacity_by_instance
                    .insert(instance, opacity);
            } else {
                self.appearance_opacity_by_instance.remove(&instance);
            }
            self.bind_appearance_surface_target(
                instance,
                frame.appearance_surface_sample_geometry(instance),
            );
            let affinity = frame
                .presentation_delta_source()
                .frame()
                .portal_presentation_affinity_for_instance(
                    instance,
                    surface,
                    self.requirement.binding(),
                );
            self.portal_motion_groups.replace_instance(
                instance,
                self.commands_by_instance.get(&instance),
                affinity,
                surface,
                self.appearance_surfaces.get(&instance).is_some(),
            );
        }
        for instance in frame.retired_appearance_instances() {
            self.appearance_opacity_by_instance.remove(instance);
            self.appearance_surfaces.remove(instance);
            if !self.has_paint_commands(*instance) {
                self.portal_motion_groups.replace_instance(
                    *instance,
                    None,
                    None,
                    self.requirement.semantic_surface(),
                    false,
                );
            }
        }
    }

    /// Unchanged geometry keeps its live slot; changed geometry starts unsampled.
    fn bind_appearance_surface_target(
        &mut self,
        instance: UiMountedInstanceIdentity,
        geometry: Option<UiMountedAppearanceSurfaceSampleGeometry>,
    ) {
        let Some(geometry) = geometry.filter(|_| !self.has_paint_commands(instance)) else {
            self.appearance_surfaces.remove(&instance);
            return;
        };
        if self
            .appearance_surfaces
            .get(&instance)
            .is_some_and(|target| target.geometry == geometry)
        {
            return;
        }
        self.appearance_surfaces.insert(
            instance,
            UiMountedAppearanceSurfaceSampleTarget {
                geometry,
                motion: UiCommandMotionAcceptance::default(),
            },
        );
    }

    pub(in crate::mounting::presentation::work_producer) fn appearance_opacity_for_command(
        &self,
        command: UiMountedPaintCommandIdentity,
    ) -> worth_ui_host_contract::UiMountedAppearanceOpacity {
        self.appearance_opacity_by_instance
            .get(&command.mounted_instance())
            .copied()
            .unwrap_or(worth_ui_host_contract::UiMountedAppearanceOpacity::ONE)
    }

    /// The surface target an instance without paint commands exposes to Motion.
    pub(in crate::mounting::presentation) fn appearance_surface_sample_target(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Option<&UiMountedAppearanceSurfaceSampleTarget> {
        if self.has_paint_commands(instance) {
            return None;
        }
        self.appearance_surfaces.get(&instance)
    }

    fn has_paint_commands(&self, instance: UiMountedInstanceIdentity) -> bool {
        self.command_identities_for_instance(instance)
            .next()
            .is_some()
    }

    /// The accepted sample of an appearance surface, lowered like a command's.
    pub(in crate::mounting::presentation::work_producer) fn appearance_surface_sample_change(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        let identity = UiMountedPaintCommandIdentity::appearance_surface(instance);
        self.command_sample_change(identity)
    }

    pub(in crate::mounting::presentation::work_producer) fn bound_appearance_surface_instances(
        &self,
    ) -> impl Iterator<Item = UiMountedInstanceIdentity> + '_ {
        self.appearance_surfaces
            .iter()
            .map(|(instance, _)| *instance)
    }

    /// Reconstruction on the same semantic surface keeps every bound surface
    /// target and its live slot; the next appearance admission rebinds geometry.
    pub(in crate::mounting::presentation::work_producer) fn inherit_appearance_surface_targets(
        &mut self,
        predecessor: &Self,
    ) {
        self.appearance_surfaces = predecessor.appearance_surfaces.clone();
    }
}

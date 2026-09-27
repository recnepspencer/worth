use std::sync::Arc;

use worth_ui_host_contract::{
    UiHostSurfacePresentationMode, UiMountedAccessibilityProjection, UiMountedDiagnosticProjection,
    UiMountedEffectFamily, UiMountedInstanceIdentity, UiSurfaceBindingGeneration,
};

use super::{UiMountedMechanicSource, UiMountedProjectionFrame, UiMountedSemanticProjection};
use crate::runtime::persistent_index::UiPersistentOrdMap;

#[derive(Clone, Default)]
pub(in crate::mounting::projection) struct UiMountedPresentationEffectSource {
    by_instance: UiPersistentOrdMap<
        UiMountedInstanceIdentity,
        (UiSurfaceBindingGeneration, Arc<[UiMountedEffectFamily]>),
    >,
    counts: UiPersistentOrdMap<(UiSurfaceBindingGeneration, UiMountedEffectFamily), usize>,
    canvas: bool,
    realtime: bool,
}

pub(super) struct UiMountedPresentationEffectCompletion<'a> {
    pub(super) semantic: &'a UiMountedSemanticProjection,
    pub(super) mechanics: &'a UiMountedMechanicSource,
    pub(super) changed: &'a [UiMountedInstanceIdentity],
    pub(super) preview: Option<&'a super::super::lowering::UiMountedPreviewProjectionInput>,
    pub(super) overlay: Option<&'a crate::mounting::UiMountedVisualOverlayProjectionInput>,
    pub(super) canvas: bool,
    pub(super) realtime: bool,
}

impl UiMountedPresentationEffectSource {
    pub(super) fn apply(&mut self, completion: UiMountedPresentationEffectCompletion<'_>) {
        for instance in completion.changed {
            self.remove(*instance);
            let Some(node) = completion.semantic.nodes.get(instance) else {
                continue;
            };
            let Some(surface) = completion
                .semantic
                .surface_for(node.receipt.semantic_surface())
            else {
                continue;
            };
            let effects = derive_instance(
                *instance,
                node,
                surface,
                completion.mechanics,
                completion.preview,
                completion.overlay,
            );
            for family in effects.iter().copied() {
                let key = (surface.binding, family);
                let next = self.counts.get(&key).copied().unwrap_or(0) + 1;
                self.counts.insert(key, next);
            }
            self.by_instance
                .insert(*instance, (surface.binding, effects));
        }
        self.canvas = completion.canvas;
        self.realtime = completion.realtime;
    }

    fn remove(&mut self, instance: UiMountedInstanceIdentity) {
        let Some((binding, effects)) = self.by_instance.get(&instance).cloned() else {
            return;
        };
        for family in effects.iter().copied() {
            let key = (binding, family);
            match self.counts.get(&key).copied() {
                Some(1) => {
                    self.counts.remove(&key);
                }
                Some(count) => self.counts.insert(key, count - 1),
                None => unreachable!("effect instance has an aggregate count"),
            }
        }
        self.by_instance.remove(&instance);
    }

    fn for_binding(
        &self,
        mode: UiHostSurfacePresentationMode,
        binding: UiSurfaceBindingGeneration,
    ) -> Box<[UiMountedEffectFamily]> {
        if mode == UiHostSurfacePresentationMode::RecordOnly {
            return vec![UiMountedEffectFamily::RecordedProjection].into_boxed_slice();
        }
        ordered_families()
            .filter(|family| match family {
                UiMountedEffectFamily::CanvasSpatial => self.canvas,
                UiMountedEffectFamily::Realtime => self.realtime,
                family @ (UiMountedEffectFamily::RecordedProjection
                | UiMountedEffectFamily::NativePaint
                | UiMountedEffectFamily::Accessibility
                | UiMountedEffectFamily::Diagnostic
                | UiMountedEffectFamily::IdentityOverlay) => {
                    self.counts.get(&(binding, *family)).is_some()
                }
            })
            .collect()
    }
}

impl UiMountedProjectionFrame {
    pub(in crate::mounting) fn presentation_effects(
        &self,
        mode: UiHostSurfacePresentationMode,
        binding: UiSurfaceBindingGeneration,
    ) -> Box<[UiMountedEffectFamily]> {
        self.presentation_effects.for_binding(mode, binding)
    }
}

fn derive_instance(
    instance: UiMountedInstanceIdentity,
    node: &super::UiMountedProjectionNodeRecord,
    surface: super::UiMountedProjectionSurface,
    mechanics: &UiMountedMechanicSource,
    preview: Option<&super::super::lowering::UiMountedPreviewProjectionInput>,
    overlay: Option<&crate::mounting::UiMountedVisualOverlayProjectionInput>,
) -> Arc<[UiMountedEffectFamily]> {
    let mut effects = Vec::new();
    push_if(
        &mut effects,
        !mechanics
            .commands_for_instance(instance, surface.surface, surface.binding)
            .is_empty()
            || preview.is_some_and(|preview| preview.mounted_instance == instance),
        UiMountedEffectFamily::NativePaint,
    );
    push_if(
        &mut effects,
        surface.audience.accessibility_disclosed()
            && matches!(
                node.receipt.accessibility(),
                UiMountedAccessibilityProjection::Admitted(_)
            ),
        UiMountedEffectFamily::Accessibility,
    );
    if surface.audience.diagnostics_disclosed() {
        if overlay.is_some_and(|overlay| {
            overlay.target_receipt.mounted_instance() == instance
                && overlay.surface == surface.surface
        }) {
            effects.push(UiMountedEffectFamily::IdentityOverlay);
        } else if matches!(
            node.receipt.diagnostic(),
            UiMountedDiagnosticProjection::Reference(_)
        ) {
            effects.push(UiMountedEffectFamily::Diagnostic);
        }
    }
    effects.into()
}

fn ordered_families() -> impl Iterator<Item = UiMountedEffectFamily> {
    [
        UiMountedEffectFamily::CanvasSpatial,
        UiMountedEffectFamily::Realtime,
        UiMountedEffectFamily::NativePaint,
        UiMountedEffectFamily::Accessibility,
        UiMountedEffectFamily::Diagnostic,
        UiMountedEffectFamily::IdentityOverlay,
    ]
    .into_iter()
}

fn push_if(effects: &mut Vec<UiMountedEffectFamily>, present: bool, family: UiMountedEffectFamily) {
    if present {
        effects.push(family);
    }
}

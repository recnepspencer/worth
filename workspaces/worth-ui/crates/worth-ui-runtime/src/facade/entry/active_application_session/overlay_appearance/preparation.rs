use super::{UiActiveOverlayAppearancePreparation, UiActiveOverlaySurfacePreparation};
use crate::runtime::{UiCommittedOverlayExtentBounds, UiMountedOverlayExtentOwner};

impl crate::facade::WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn prepare_overlay_appearance_sources(
        &self,
    ) -> Result<UiActiveOverlayAppearancePreparation, ()> {
        let owners = self
            .authored_overlay_bindings
            .bound_owners()
            .map(|(declaration, runtime, bindings)| (declaration, runtime, bindings.clone()))
            .collect();
        self.prepare_overlay_appearance_sources_from(owners, None, None)
    }

    pub(in crate::facade::entry) fn prepare_overlay_appearance_sources_for_portal_transition(
        &self,
        transition: &crate::runtime::portal::UiPreparedPortalServiceTransition,
        stage: Option<&crate::runtime::portal::UiPortalOverlayBindingStage>,
        staged_motion: Option<crate::runtime::motion::UiMotionOverlayOwnerRow>,
        retain_exit: bool,
    ) -> Result<UiActiveOverlayAppearancePreparation, ()> {
        let owners = self
            .authored_overlay_bindings
            .candidate_bound_owners(transition, stage, retain_exit)
            .map_err(|_| ())?;
        self.prepare_overlay_appearance_sources_from(
            owners,
            Some((transition, staged_motion, retain_exit)),
            None,
        )
    }

    pub(in crate::facade::entry) fn prepare_replacement_overlay_appearance_sources(
        &self,
        authority: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        mounted: &crate::mounting::UiMountedGraphReplacementSuccessor,
        bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    ) -> Result<UiActiveOverlayAppearancePreparation, ()> {
        let owners = bindings
            .bound_owners()
            .map(|(declaration, runtime, bindings)| (declaration, runtime, bindings.clone()))
            .collect();
        self.prepare_overlay_appearance_sources_from(owners, None, Some((authority, mounted)))
    }

    fn prepare_overlay_appearance_sources_from(
        &self,
        bound_owners: Vec<(
            worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            crate::runtime::portal::UiPortalOverlayBindingOwner,
        )>,
        portal_transition: Option<(
            &crate::runtime::portal::UiPreparedPortalServiceTransition,
            Option<crate::runtime::motion::UiMotionOverlayOwnerRow>,
            bool,
        )>,
        replacement: Option<(
            &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
            &crate::mounting::UiMountedGraphReplacementSuccessor,
        )>,
    ) -> Result<UiActiveOverlayAppearancePreparation, ()> {
        if bound_owners.is_empty() {
            return Ok(UiActiveOverlayAppearancePreparation {
                surfaces: Box::new([]),
            });
        }
        let generation = replacement
            .map_or_else(
                || self.generation_identity(),
                |(authority, _)| authority.generation_identity(),
            )
            .clone();
        let mut surfaces = Vec::new();
        let material = replacement.map_or_else(
            || self.application.authored_overlay_material(),
            |(authority, _)| authority.authored_overlay_material(),
        );
        for (declaration, runtime, bindings) in bound_owners {
            let backdrops = material
                .backdrop_declarations_for_surface(declaration)
                .map(|declaration| (declaration.identity(), declaration.clone()))
                .collect::<std::collections::BTreeMap<_, _>>();
            let regions = backdrops
                .values()
                .filter_map(|backdrop| match backdrop.extent() {
                    worth_ui_dsl::UiBackdropExtentBasis::PresentedMosaicRegion {
                        region, ..
                    } => Some(region),
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<_>>();
            let extent =
                self.mounted
                    .current_surface_viewport(runtime)
                    .and_then(|(revision, viewport)| {
                        let viewport = UiCommittedOverlayExtentBounds::new(
                            viewport.x(),
                            viewport.y(),
                            viewport.width(),
                            viewport.height(),
                        )
                        .ok()?;
                        let mut mounted_regions = Vec::new();
                        for region in &regions {
                            let Some(rows) = (match replacement {
                                Some((_, mounted)) => {
                                    mounted.current_region_extents(runtime, &generation, *region)
                                }
                                None => self.mounted.current_region_extents(
                                    runtime,
                                    &generation,
                                    *region,
                                ),
                            }) else {
                                continue;
                            };
                            for (occurrence, bounds) in rows {
                                mounted_regions.push(
                                    crate::runtime::UiMountedOverlayRegionExtent::new(
                                        *region,
                                        occurrence,
                                        UiCommittedOverlayExtentBounds::new(
                                            bounds.x(),
                                            bounds.y(),
                                            bounds.width(),
                                            bounds.height(),
                                        )
                                        .ok()?,
                                    ),
                                );
                            }
                        }
                        UiMountedOverlayExtentOwner::new(
                            generation.clone(),
                            declaration,
                            runtime,
                            revision.get(),
                            viewport,
                            mounted_regions,
                        )
                        .ok()
                    });
            surfaces.push(UiActiveOverlaySurfacePreparation {
                declaration_surface: declaration,
                runtime_surface: runtime,
                portal_stack: match portal_transition {
                    Some((transition, _, retain_exit)) => Some(
                        self.portal
                            .as_ref()
                            .ok_or(())?
                            .surface_stack_snapshot(runtime)
                            .for_transition(runtime, transition, retain_exit)
                            .ok_or(())?,
                    ),
                    None => None,
                },
                staged_motion: portal_transition.and_then(|(_, motion, _)| motion),
                extent,
                bindings,
                backdrops,
            });
        }
        Ok(UiActiveOverlayAppearancePreparation {
            surfaces: surfaces.into_boxed_slice(),
        })
    }
}

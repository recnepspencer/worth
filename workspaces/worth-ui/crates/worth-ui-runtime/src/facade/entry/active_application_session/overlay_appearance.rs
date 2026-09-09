use crate::runtime::{UiCommittedOverlayExtentBounds, UiMountedOverlayExtentOwner};

#[path = "overlay_appearance/backdrop_projection.rs"]
mod backdrop_projection;
#[path = "overlay_appearance/local_sources.rs"]
mod local_sources;
#[path = "overlay_appearance/owner_state.rs"]
mod owner_state;
use backdrop_projection::lower_surface;
pub(in crate::facade::entry) use owner_state::UiActiveOverlayCompositionOwners;
use owner_state::{UiActiveOverlayOwnerUpdate, UiActiveOverlayRetentionCandidate};

pub(in crate::facade::entry) struct UiActiveOverlayAppearancePreparation {
    surfaces: Box<[UiActiveOverlaySurfacePreparation]>,
}

struct UiActiveOverlaySurfacePreparation {
    declaration_surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    portal_stack: Option<crate::runtime::portal::UiPortalSurfaceStackSnapshot>,
    staged_motion: Option<crate::runtime::motion::UiMotionOverlayOwnerRow>,
    extent: Option<UiMountedOverlayExtentOwner>,
    bindings: crate::runtime::portal::UiPortalOverlayBindingOwner,
    backdrops: std::collections::BTreeMap<
        worth_ui_dsl::UiBackdropIdentity,
        worth_ui_dsl::UiBackdropDeclaration,
    >,
}

impl super::WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn prepare_overlay_appearance_sources(
        &self,
    ) -> Result<UiActiveOverlayAppearancePreparation, ()> {
        let owners = self
            .authored_overlay_bindings
            .bound_owners()
            .map(|(declaration, runtime, bindings)| (declaration, runtime, bindings.clone()))
            .collect();
        self.prepare_overlay_appearance_sources_from(owners, None)
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
        )
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
    ) -> Result<UiActiveOverlayAppearancePreparation, ()> {
        let generation = self.generation_identity().clone();
        let mut surfaces = Vec::new();
        let material = self.application.authored_overlay_material();
        if bound_owners.is_empty() {
            return Ok(UiActiveOverlayAppearancePreparation {
                surfaces: surfaces.into_boxed_slice(),
            });
        }
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
                            let Some(rows) =
                                self.mounted
                                    .current_region_extents(runtime, &generation, *region)
                            else {
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

impl UiActiveOverlayAppearancePreparation {
    pub(in crate::facade::entry) fn lower(
        &self,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        requested_surfaces: &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        owners: &mut UiActiveOverlayCompositionOwners,
        generation: &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        portal: Option<&crate::runtime::portal::UiPortalRuntimeState>,
        motion: Option<&crate::runtime::motion::UiMotionRuntimeState>,
        presentation: &crate::runtime::presentation_state::UiApplicationPresentationState,
        capabilities: &crate::capability::CapabilitySnapshot,
        appearance: Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    ) -> Result<Vec<crate::mounting::UiMountedAppearanceSurfaceOverlayInput>, ()> {
        let presentation_export = presentation.overlay_owner_export(generation.clone(), attempt);
        let declaration_revision = generation
            .semantic_package_identity()
            .narrowing_fingerprint()
            .max(1);
        let requested_surfaces = requested_surfaces
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let mut active_surfaces = std::collections::BTreeSet::new();
        let mut outputs = Vec::new();
        let mut updates = Vec::new();
        let mut backdrop_candidates = std::collections::BTreeMap::new();
        for surface in &self.surfaces {
            let runtime_surface = surface.runtime_surface;
            if !requested_surfaces.contains(&runtime_surface)
                || (surface.backdrops.is_empty() && surface.bindings.is_empty())
            {
                continue;
            }
            active_surfaces.insert(runtime_surface);
            let extent = surface.extent.as_ref().ok_or(())?;
            let current = owners
                .current()
                .get(&runtime_surface)
                .and_then(|owner| owner.current());
            let mut source_work = owner_state::UiActiveBackdropAppearanceWork::default();
            let previous_sources = owners.sources(runtime_surface);
            let (observed_sources, changed_bindings) =
                local_sources::capture(portal, motion, surface, previous_sources, &mut source_work);
            let (portal_snapshot, portal_changes) = local_sources::localized_portal_source(
                &observed_sources,
                previous_sources,
                current,
                &changed_bindings,
                &mut source_work,
            )?;
            let (motion_export, motion_changes) = local_sources::localized_motion_source(
                &observed_sources,
                previous_sources,
                current,
                &changed_bindings,
                runtime_surface,
                &mut source_work,
            )?;
            let sources = crate::runtime::overlay_composition::UiOverlayOwnerSources::from_exports(
                generation,
                portal_snapshot,
                extent,
                &presentation_export,
                &surface.bindings,
                motion_export,
            );
            let replace_owner = owners.current().get(&runtime_surface).is_none_or(|owner| {
                owner.current().is_none_or(|current| {
                    current.generation()
                        != &crate::runtime::overlay_composition::UiOverlayApplicationGeneration::from_prepared(generation.clone())
                        || current.backdrop_declaration_revision() != declaration_revision
                        || current.declaration_surface() != surface.declaration_surface
                })
            });
            let update = if replace_owner {
                let owner = crate::runtime::overlay_composition::
                    UiOverlayCompositionOwnerLifecycle::admit_from_owners(
                        surface.backdrops.values().cloned(), declaration_revision, sources,
                    ).map_err(|_| ())?;
                UiActiveOverlayOwnerUpdate::Replace {
                    surface: runtime_surface,
                    owner,
                }
            } else {
                let owner = owners.current().get(&runtime_surface).ok_or(())?;
                let changes = overlay_changes(
                    owner.current().ok_or(())?,
                    surface,
                    extent,
                    portal_changes,
                    motion_changes,
                );
                let prepared = owner
                    .prepare_successor_from_owners(sources, &changes)
                    .map_err(|_| ())?;
                UiActiveOverlayOwnerUpdate::Advance {
                    surface: runtime_surface,
                    prepared,
                }
            };
            let snapshot = update.snapshot().ok_or(())?;
            let mut candidate = if snapshot.participants().is_empty() {
                owner_state::UiActiveBackdropAppearanceCandidate {
                    projections: Default::default(),
                    work: source_work,
                    sources: None,
                }
            } else {
                let previous = (!replace_owner)
                    .then(|| owners.backdrop_projections(runtime_surface))
                    .flatten();
                let (output, candidate) = lower_surface(
                    snapshot,
                    surface,
                    presentation,
                    capabilities,
                    appearance,
                    previous,
                    source_work,
                )?;
                outputs.push(output);
                candidate
            };
            candidate.sources = Some(observed_sources);
            backdrop_candidates.insert(runtime_surface, candidate);
            updates.push(update);
        }
        owners.stage(
            attempt,
            UiActiveOverlayRetentionCandidate::new(
                requested_surfaces,
                active_surfaces,
                updates,
                backdrop_candidates,
            ),
        )?;
        Ok(outputs)
    }
}

fn overlay_changes(
    current: &crate::runtime::overlay_composition::UiOverlayStackSnapshot,
    surface: &UiActiveOverlaySurfacePreparation,
    extent: &UiMountedOverlayExtentOwner,
    portal_changes: Vec<worth_ui_dsl::UiPortalDeclarationId>,
    motion_changes: Vec<worth_ui_dsl::UiPortalDeclarationId>,
) -> crate::runtime::overlay_composition::UiOverlayChangeSet {
    use crate::runtime::overlay_composition::UiOverlayChangedBasis;
    let mut changes = Vec::new();
    changes.extend(
        portal_changes
            .into_iter()
            .map(UiOverlayChangedBasis::Portal),
    );
    let next_extent = extent.export();
    if current.extent_revision() != next_extent.revision() {
        let regions = next_extent
            .regions()
            .iter()
            .map(|row| (row.identity(), row.bounds()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let viewport = next_extent.viewport();
        let mut changed_regions = std::collections::BTreeSet::new();
        let mut viewport_changed = false;
        for row in current.participants() {
            let crate::runtime::overlay_composition::UiOverlayStackParticipant::Backdrop(row) = row
            else {
                continue;
            };
            let previous = row.extent().bounds();
            if matches!(
                row.extent().basis(),
                worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(_)
            ) {
                viewport_changed |= (
                    viewport.x(),
                    viewport.y(),
                    viewport.width(),
                    viewport.height(),
                ) != (
                    previous.x(),
                    previous.y(),
                    previous.width(),
                    previous.height(),
                );
            }
            if let worth_ui_dsl::UiBackdropExtentBasis::PresentedMosaicRegion { region, .. } =
                row.extent().basis()
            {
                let changed = regions.get(&region).is_none_or(|bounds| {
                    (bounds.x(), bounds.y(), bounds.width(), bounds.height())
                        != (
                            previous.x(),
                            previous.y(),
                            previous.width(),
                            previous.height(),
                        )
                });
                if changed {
                    changed_regions.insert(region);
                }
            }
        }
        if viewport_changed {
            changes.push(UiOverlayChangedBasis::SurfaceExtent(
                surface.declaration_surface,
            ));
        }
        changes.extend(
            changed_regions
                .into_iter()
                .map(UiOverlayChangedBasis::RegionExtent),
        );
    }
    changes.extend(
        motion_changes
            .into_iter()
            .map(UiOverlayChangedBasis::PortalMotion),
    );
    crate::runtime::overlay_composition::UiOverlayChangeSet::from_changes(changes)
}

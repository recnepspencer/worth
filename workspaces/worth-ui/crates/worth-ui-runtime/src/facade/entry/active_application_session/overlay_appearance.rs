use crate::runtime::UiMountedOverlayExtentOwner;

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
    /// Scroll chrome derived alongside the overlays, so one closure resolves
    /// every non-node paint against the attempt's theme binding.
    scroll_chrome: Vec<super::scroll_chrome_appearance::UiActiveScrollChromeSurfacePreparation>,
    scroll_motion:
        Vec<crate::mounting::presentation::work_producer::UiMountedScrollMotionGroupInput>,
    scroll_geometry_reservations:
        std::collections::BTreeMap<worth_ui_host_contract::UiSemanticSurfaceIdentity, usize>,
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

#[path = "overlay_appearance/preparation.rs"]
mod preparation;
#[path = "overlay_appearance/scroll_motion.rs"]
mod scroll_motion;

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
        prepared_binding: Option<&crate::runtime::appearance::UiActiveThemeBinding>,
    ) -> Result<crate::mounting::UiMountedAppearanceDerivedInput, ()> {
        self.lower_with_themes(
            attempt,
            requested_surfaces,
            owners,
            generation,
            portal,
            motion,
            presentation,
            capabilities,
            appearance,
            None,
            prepared_binding,
        )
    }

    /// Everything this attempt paints that no node authored: the overlays
    /// the portal and backdrop world composed, and the scroll chrome the
    /// accepted pose derived. Both resolve against the same theme binding.
    pub(in crate::facade::entry) fn lower_with_themes(
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
        themes: Option<
            &crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        >,
        prepared_binding: Option<&crate::runtime::appearance::UiActiveThemeBinding>,
    ) -> Result<crate::mounting::UiMountedAppearanceDerivedInput, ()> {
        let overlays = self.lower_overlays(
            attempt,
            requested_surfaces,
            owners,
            generation,
            portal,
            motion,
            presentation,
            capabilities,
            appearance,
            themes,
            prepared_binding,
        )?;
        let scroll_chrome = super::scroll_chrome_appearance::lower_scroll_chrome_appearance(
            &self.scroll_chrome,
            requested_surfaces,
            presentation,
            capabilities,
            appearance,
            themes,
            prepared_binding,
        )
        .map_err(|_| ())?;
        Ok(crate::mounting::UiMountedAppearanceDerivedInput {
            scroll_geometry_reservations: self
                .scroll_geometry_reservations
                .iter()
                .filter(|(surface, _)| requested_surfaces.contains(surface))
                .map(|(surface, bytes)| (*surface, *bytes))
                .collect(),
            overlays,
            scroll_chrome,
            scroll_motion: self
                .scroll_motion
                .iter()
                .filter(|group| requested_surfaces.contains(&group.target.semantic_surface()))
                .cloned()
                .collect(),
        })
    }

    fn lower_overlays(
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
        themes: Option<
            &crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        >,
        prepared_binding: Option<&crate::runtime::appearance::UiActiveThemeBinding>,
    ) -> Result<Vec<crate::mounting::UiMountedAppearanceSurfaceOverlayInput>, ()> {
        if self.surfaces.is_empty() && owners.is_empty() {
            return Ok(Vec::new());
        }
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
                    themes,
                    prepared_binding,
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
        // An extent revision still advances when no active Backdrop exposes a
        // changed rectangle. Carry that owner change even when its dependency
        // scope is empty; the consumer must not infer reuse across revisions.
        if viewport_changed || changed_regions.is_empty() {
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

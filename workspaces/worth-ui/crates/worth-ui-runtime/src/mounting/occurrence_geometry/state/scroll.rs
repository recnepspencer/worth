use super::*;
use crate::mounting::presentation::{UiPublishedRect, UiScrollPoseShift};
use crate::mounting::projection::UiMountedAppearanceClip;
use crate::mounting::{UiHitAncestorClip, UiHitScrollMove};

impl UiMountedOccurrenceGeometryState {
    /// The owner of the region at `slot` of `target`'s chain, and the
    /// region's boxes where it is laid out. The slot is the Scroll owner's
    /// validated ownership-chain slot, paired with the clip bindings by
    /// mounted geometry admission.
    pub(crate) fn scroll_region_geometry(
        &self,
        surface: UiSemanticSurfaceIdentity,
        target: UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<(
        UiMountedInstanceIdentity,
        crate::mounting::UiLaidOut<crate::mounting::UiMountedScrollRegionBoxes>,
    )> {
        let geometry = self.surfaces.get(&surface)?;
        let target = geometry.occurrences.get(&target)?;
        let super::super::UiMountedScrollClipBinding::Region { owner, declaration } =
            target.scroll_clip_bindings.as_ref().ok()?.get(slot)?
        else {
            return None;
        };
        let content = geometry.occurrences.get(owner)?.bounds;
        let (_, slot) = geometry
            .scroll_index
            .regions(*owner)
            .iter()
            .find(|(candidate, _)| candidate == declaration)?;
        let viewport = geometry.regions.get(declaration)?.get(*slot)?.2;
        Some((
            *owner,
            crate::mounting::UiLaidOut::from_layout(
                crate::mounting::UiMountedScrollRegionBoxes::new(content, viewport),
            ),
        ))
    }

    /// The Scroll region owner whose offset moves `instance`, when `instance`
    /// is content laid out relative to such an owner rather than an owner
    /// itself. Scrolled content is not the region's graph descendant, so its
    /// own ownership chain never names the region; the layout parent link is
    /// what ties it to the offset it travels with.
    pub(crate) fn scrolled_content_owner(
        &self,
        surface: UiSemanticSurfaceIdentity,
        instance: UiMountedInstanceIdentity,
    ) -> Option<UiMountedInstanceIdentity> {
        let geometry = self.surfaces.get(&surface)?;
        let owns_region = |candidate: UiMountedInstanceIdentity| {
            !geometry.scroll_index.regions(candidate).is_empty()
        };
        let mut cursor = geometry.occurrences.get(&instance)?.parent;
        while let Some(ancestor) = cursor {
            if owns_region(ancestor) {
                return Some(ancestor);
            }
            cursor = geometry.occurrences.get(&ancestor)?.parent;
        }
        None
    }

    /// The Scroll region owner a gesture over `instance` addresses:
    /// `instance` itself when it owns a region, even one that is in turn
    /// scrolled content of another, otherwise the owner it travels with.
    pub(crate) fn addressed_scroll_owner(
        &self,
        surface: UiSemanticSurfaceIdentity,
        instance: UiMountedInstanceIdentity,
    ) -> Option<UiMountedInstanceIdentity> {
        let geometry = self.surfaces.get(&surface)?;
        if !geometry.scroll_index.regions(instance).is_empty() {
            return Some(instance);
        }
        self.scrolled_content_owner(surface, instance)
    }

    /// The offset the displayed pose of one region occurrence was last built
    /// from. This is the displayed truth rather than the semantic target: the
    /// accepted-sample settlement writes it, so a region still traveling
    /// toward a new offset reports the one the host has already presented.
    pub(crate) fn applied_scroll_pose(
        &self,
        surface: UiSemanticSurfaceIdentity,
        owner_instance: UiMountedInstanceIdentity,
    ) -> Option<crate::runtime::scroll::UiScrollOffset> {
        self.surfaces
            .get(&surface)?
            .scroll_poses
            .get(&owner_instance)
            .copied()
    }

    /// Prepare `poses` over this surface's mounted geometry. Hit rows move
    /// from `shown`, the offset each listed owner stands at in the rows of
    /// the frame the host shows; an owner `shown` omits stands there where
    /// mounted geometry holds it. Geometry staged past what the host shows
    /// moves from where it is staged, and hit rows from where they committed.
    pub(crate) fn prepare_scroll_pose(
        &self,
        surface: UiSemanticSurfaceIdentity,
        poses: &[(
            UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
        shown: &[(
            UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<UiPreparedMountedScrollPose, UiMountedOccurrenceGeometryDenial> {
        let geometry = self
            .surfaces
            .get(&surface)
            .ok_or(UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?;
        let mut translations = BTreeMap::<UiMountedInstanceIdentity, PoseMove>::new();
        let mut work = crate::mounting::UiHitTestSpatialWork::default();
        for (owner, offset) in poses {
            if !geometry.occurrences.contains_key(owner) {
                return Err(UiMountedOccurrenceGeometryDenial::UnknownMountedInstance);
            }
            let previous = geometry
                .scroll_poses
                .get(owner)
                .copied()
                .unwrap_or_default();
            let committed = shown
                .iter()
                .find(|(shown, _)| shown == owner)
                .map_or(previous, |(_, committed)| *committed);
            if previous == *offset && committed == *offset {
                continue;
            }
            let shift = PoseMove {
                geometry: UiScrollPoseShift::between(previous, *offset),
                hits: UiScrollPoseShift::between(committed, *offset),
            };
            for instance in geometry.scroll_index.descendants(*owner) {
                work.scroll_geometry_members_visited += 1;
                let translation = translations.entry(*instance).or_insert(PoseMove {
                    geometry: UiScrollPoseShift::none(),
                    hits: UiScrollPoseShift::none(),
                });
                *translation = PoseMove {
                    geometry: translation.geometry.then(shift.geometry),
                    hits: translation.hits.then(shift.hits),
                };
            }
        }
        let mut rows = Vec::with_capacity(translations.len());
        let mut moved = Vec::with_capacity(translations.len());
        let mut changes_coverage = false;
        for (instance, shift) in &translations {
            let Some(row) = geometry.occurrences.get(instance) else {
                continue;
            };
            let suppressed = suppresses(row);
            let mut row = row.clone();
            row.bounds = UiPublishedRect::box_following_pose(row.bounds, shift.geometry);
            for (binding, clip) in row
                .mosaic_clip_bindings
                .iter()
                .zip(row.mosaic_clips.iter_mut())
            {
                if let super::super::UiMountedMosaicClipBinding::Region { owner, .. } = binding {
                    if let Some(shift) = translations.get(owner) {
                        *clip = UiPublishedRect::box_following_pose(*clip, shift.geometry);
                    }
                }
            }
            if let (Ok(bindings), Ok(clips)) = (&row.scroll_clip_bindings, &mut row.scroll_clips) {
                for (binding, clip) in bindings.iter().zip(clips.iter_mut()) {
                    let moving = match binding {
                        super::super::UiMountedScrollClipBinding::Region { owner, .. }
                        | super::super::UiMountedScrollClipBinding::Occurrence(owner) => {
                            translations.get(owner)
                        }
                        super::super::UiMountedScrollClipBinding::Viewport => None,
                    };
                    if let Some(shift) = moving {
                        *clip = UiPublishedRect::box_following_pose(*clip, shift.geometry);
                    }
                }
            }
            changes_coverage |= suppresses(&row) != suppressed;
            moved.push((
                *instance,
                UiHitScrollMove::new(
                    shift.hits,
                    UiHitAncestorClip::relative_to(ancestors(&row), row.bounds),
                ),
            ));
            rows.push((*instance, row));
        }
        let mut regions = Vec::new();
        for (instance, shift) in &translations {
            for (declaration, index) in geometry.scroll_index.regions(*instance) {
                work.scroll_geometry_regions_visited += 1;
                let bounds = geometry.regions[declaration][*index].2;
                regions.push((
                    *declaration,
                    *index,
                    UiPublishedRect::box_following_pose(bounds, shift.geometry),
                ));
            }
        }
        Ok(UiPreparedMountedScrollPose {
            surface,
            poses: poses.to_vec(),
            rows,
            regions,
            translations: moved,
            changes_coverage,
            work,
        })
    }

    pub(crate) fn restore_scroll_geometry(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        identity: &crate::mounting::UiMountedIdentityState,
        scroll: &mut crate::runtime::scroll::UiScrollRuntimeState,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedOccurrenceGeometryDenial> {
        let binding = identity
            .projection_surface(surface)
            .ok_or(UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?
            .0
            .binding_generation();
        let mut poses = BTreeMap::new();
        // Gathered before the walk: each restored owner is reconciled back
        // into the same Scroll state the instances came from.
        for target in scroll.ownership_instances().collect::<Vec<_>>() {
            let Ok(chain) = scroll.ownership_chain(target).cloned() else {
                continue;
            };
            for (slot, owner) in chain.owners().iter().copied().enumerate() {
                if owner.semantic_surface() != surface {
                    continue;
                }
                let Some((mounted, region)) = self.scroll_region_geometry(surface, target, slot)
                else {
                    continue;
                };
                let instance = identity
                    .projection_instance(mounted)
                    .ok_or(UiMountedOccurrenceGeometryDenial::UnknownMountedInstance)?;
                let incarnation =
                    crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                        instance.mount_incarnation(),
                    );
                let bounds = region
                    .in_layout_space()
                    .bounds()
                    .ok_or(UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry)?;
                let registration = crate::runtime::scroll::UiScrollOwnerRegistration::new(
                    owner,
                    incarnation,
                    bounds.axes(),
                    bounds,
                    crate::runtime::scroll::UiScrollOffset::origin(),
                );
                let (anchor, policy) = self.scroll_rebind_anchor(
                    surface,
                    mounted,
                    binding,
                    scroll.owner_anchor(owner, incarnation).ok().flatten(),
                    // An owner with no record and an owner whose record belongs
                    // to an earlier incarnation are the same thing to a restore:
                    // neither names an offset this surface has traveled, so both
                    // start it at rest rather than at a distance nothing here can
                    // account for.
                    scroll
                        .offset(owner, incarnation)
                        .unwrap_or_else(|_| crate::runtime::scroll::UiScrollOffset::origin()),
                    bounds,
                );
                scroll
                    .reconcile_rebind(crate::runtime::scroll::UiScrollRebindRequest::new(
                        registration,
                        anchor,
                        policy,
                    ))
                    .expect("origin is within validated region extents");
                poses.insert(
                    mounted,
                    scroll
                        .offset(owner, incarnation)
                        .expect("prepared region owner"),
                );
            }
        }
        let prepared =
            self.prepare_scroll_pose(surface, &poses.into_iter().collect::<Vec<_>>(), &[])?;
        let changed = prepared.changed_instances();
        self.apply_scroll_pose(prepared);
        Ok(changed)
    }

    pub(crate) fn apply_scroll_pose(&mut self, prepared: UiPreparedMountedScrollPose) {
        let geometry = self
            .surfaces
            .get_mut(&prepared.surface)
            .expect("prepared current surface");
        geometry.scroll_poses.extend(prepared.poses);
        for (instance, row) in prepared.rows {
            geometry.occurrences.insert(instance, row);
        }
        for (declaration, index, bounds) in prepared.regions {
            geometry
                .regions
                .get_mut(&declaration)
                .expect("prepared region")[index]
                .2 = bounds;
        }
    }
}

/// How far a pose moves one descendant: its mounted geometry from where
/// mounted geometry holds the pose, and its hit rows from where the rows the
/// host shows committed it.
#[derive(Clone, Copy)]
struct PoseMove {
    geometry: UiScrollPoseShift,
    hits: UiScrollPoseShift,
}

/// The coverage an occurrence's ancestor clips leave it, read as clip
/// derivation reads them. An unresolved Scroll clip clips nothing.
fn ancestors(row: &UiMountedOccurrenceGeometryRow) -> UiMountedAppearanceClip {
    row.scroll_clips
        .as_deref()
        .map_or(UiMountedAppearanceClip::Unclipped, |scroll| {
            crate::mounting::projection::ancestor_clip(
                row.mosaic_clips.iter().chain(scroll).copied(),
            )
        })
}

/// Whether an occurrence's ancestor clips suppress it.
fn suppresses(row: &UiMountedOccurrenceGeometryRow) -> bool {
    ancestors(row) == UiMountedAppearanceClip::Suppressed
}

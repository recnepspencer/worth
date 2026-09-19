use super::*;

impl UiMountedOccurrenceGeometryState {
    /// The slot is the Scroll owner's validated ownership-chain slot, paired
    /// with the clip bindings by mounted geometry admission.
    pub(crate) fn scroll_region_geometry(
        &self,
        surface: UiSemanticSurfaceIdentity,
        target: UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<(
        UiMountedInstanceIdentity,
        UiMountedCanonicalBox,
        UiMountedCanonicalBox,
    )> {
        let geometry = self.surfaces.get(&surface)?;
        let target = geometry.occurrences.get(&target)?;
        let super::super::UiMountedScrollClipBinding::Region { owner, declaration } =
            target.scroll_clip_bindings.as_ref().ok()?.get(slot)?
        else {
            return None;
        };
        let content = geometry.occurrences.get(owner)?.bounds;
        let viewport = geometry
            .regions
            .get(declaration)?
            .iter()
            .find(|row| row.0 == *owner)?
            .2;
        Some((*owner, content, viewport))
    }

    pub(crate) fn prepare_scroll_pose(
        &self,
        surface: UiSemanticSurfaceIdentity,
        poses: &[(
            UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<UiPreparedMountedScrollPose, UiMountedOccurrenceGeometryDenial> {
        let geometry = self
            .surfaces
            .get(&surface)
            .ok_or(UiMountedOccurrenceGeometryDenial::MissingSurfaceBinding)?;
        let scale = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
        let mut translations = BTreeMap::<UiMountedInstanceIdentity, (f32, f32)>::new();
        for (owner, offset) in poses {
            if !geometry.occurrences.contains_key(owner) {
                return Err(UiMountedOccurrenceGeometryDenial::UnknownMountedInstance);
            }
            let previous = geometry
                .scroll_poses
                .get(owner)
                .copied()
                .unwrap_or_default();
            if previous == *offset {
                continue;
            }
            let dx =
                ((previous.inline_subpixels() - offset.inline_subpixels()) as f64 / scale) as f32;
            let dy =
                ((previous.block_subpixels() - offset.block_subpixels()) as f64 / scale) as f32;
            let mut pending = geometry.children.get(owner).cloned().unwrap_or_default();
            while let Some(instance) = pending.pop() {
                let translation = translations.entry(instance).or_default();
                translation.0 += dx;
                translation.1 += dy;
                if let Some(children) = geometry.children.get(&instance) {
                    pending.extend(children);
                }
            }
        }
        let mut rows = Vec::with_capacity(translations.len());
        for (instance, (dx, dy)) in &translations {
            let Some(row) = geometry.occurrences.get(instance) else {
                continue;
            };
            let mut row = row.clone();
            row.bounds = translate(row.bounds, *dx, *dy)?;
            for (binding, clip) in row
                .mosaic_clip_bindings
                .iter()
                .zip(row.mosaic_clips.iter_mut())
            {
                if let super::super::UiMountedMosaicClipBinding::Region { owner, .. } = binding {
                    if let Some((dx, dy)) = translations.get(owner) {
                        *clip = translate(*clip, *dx, *dy)?;
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
                    if let Some((dx, dy)) = moving {
                        *clip = translate(*clip, *dx, *dy)?;
                    }
                }
            }
            rows.push((*instance, row));
        }
        let mut regions = Vec::new();
        for (declaration, occurrences) in &geometry.regions {
            for (index, (instance, _, bounds)) in occurrences.iter().enumerate() {
                if let Some((dx, dy)) = translations.get(instance) {
                    regions.push((*declaration, index, translate(*bounds, *dx, *dy)?));
                }
            }
        }
        Ok(UiPreparedMountedScrollPose {
            surface,
            poses: poses.to_vec(),
            rows,
            regions,
        })
    }

    pub(crate) fn restore_scroll_geometry(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        identity: &crate::mounting::UiMountedIdentityState,
        scroll: &mut crate::runtime::scroll::UiScrollRuntimeState,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedOccurrenceGeometryDenial> {
        let mut poses = BTreeMap::new();
        for target in scroll.ownership_instances() {
            let Ok(chain) = scroll.ownership_chain(target).cloned() else {
                continue;
            };
            for (slot, owner) in chain.owners().iter().copied().enumerate() {
                if owner.semantic_surface() != surface {
                    continue;
                }
                let Some((mounted, content, viewport)) =
                    self.scroll_region_geometry(surface, target, slot)
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
                let bounds =
                    crate::runtime::scroll::UiScrollBounds::from_mounted_region(content, viewport)
                        .ok_or(UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry)?;
                let registration = crate::runtime::scroll::UiScrollOwnerRegistration::new(
                    owner,
                    incarnation,
                    bounds.axes(),
                    bounds,
                    crate::runtime::scroll::UiScrollOffset::origin(),
                );
                scroll
                    .reconcile_rebind(crate::runtime::scroll::UiScrollRebindRequest::new(
                        registration,
                        None,
                        crate::runtime::scroll::UiScrollAnchorPolicy::Clamp,
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
        let prepared = self.prepare_scroll_pose(surface, &poses.into_iter().collect::<Vec<_>>())?;
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

pub(crate) struct UiPreparedMountedScrollPose {
    surface: UiSemanticSurfaceIdentity,
    poses: Vec<(
        UiMountedInstanceIdentity,
        crate::runtime::scroll::UiScrollOffset,
    )>,
    rows: Vec<(UiMountedInstanceIdentity, UiMountedOccurrenceGeometryRow)>,
    regions: Vec<(
        worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        usize,
        UiMountedCanonicalBox,
    )>,
}

impl UiPreparedMountedScrollPose {
    pub(crate) fn changed_instances(&self) -> Box<[UiMountedInstanceIdentity]> {
        self.rows.iter().map(|row| row.0).collect()
    }
}

fn translate(
    bounds: UiMountedCanonicalBox,
    dx: f32,
    dy: f32,
) -> Result<UiMountedCanonicalBox, UiMountedOccurrenceGeometryDenial> {
    UiMountedCanonicalBox::canonicalize(worth_ui_host_contract::UiMountedCanonicalBoxInput {
        x: bounds.x() + dx,
        y: bounds.y() + dy,
        width: bounds.width(),
        height: bounds.height(),
        coordinate_space: bounds.coordinate_space(),
    })
    .map_err(|_| UiMountedOccurrenceGeometryDenial::ParentCoordinateSpaceMismatch)
}

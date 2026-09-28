//! Exact mounted allocation and ancestor clip lookup.
use worth_ui_host_contract::{UiMountedAllocationProjection, UiMountedCanonicalBox};

impl super::UiMountedOccurrenceGeometryState {
    pub(crate) fn projection(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> Result<
        Option<UiMountedAllocationProjection>,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        Ok(self
            .projection_on_grid(instance)?
            .map(|(projection, _)| projection))
    }

    /// The allocation `projection` paints, and the box on record when the
    /// device grid painted it elsewhere. Hit testing reads the box on record.
    pub(crate) fn projection_on_grid(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> Result<
        Option<(UiMountedAllocationProjection, Option<UiMountedCanonicalBox>)>,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        let surface = instance.basis().semantic_surface_identity();
        let Some(surface_geometry) = self.surfaces.get(&surface) else {
            return Ok(None);
        };
        let row = surface_geometry
            .occurrences
            .get(&instance.identity())
            .ok_or(crate::mounting::UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry)?;
        if row.incarnation != instance.mount_incarnation() {
            return Err(
                crate::mounting::UiMountedOccurrenceGeometryDenial::StaleOccurrenceGeometry,
            );
        }
        // The box on record is where the accepted offset put it. What is
        // painted is that box moved onto the device grid, derived here and
        // written nowhere, so the offset behind it keeps its precision.
        let (bounds, recorded) =
            match self.presented_scroll_grid_correction(surface, instance.identity())? {
                Some(correction) => (correction.onto_grid(row.bounds)?, Some(row.bounds)),
                None => (row.bounds, None),
            };
        Ok(Some((
            UiMountedAllocationProjection::Known {
                bounds,
                basis: row.basis,
            },
            recorded,
        )))
    }

    pub(crate) fn mosaic_clips(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> &[UiMountedCanonicalBox] {
        self.surfaces
            .get(&instance.basis().semantic_surface_identity())
            .and_then(|surface| surface.occurrences.get(&instance.identity()))
            .map_or(&[], |row| row.mosaic_clips.as_ref())
    }

    pub(crate) fn scroll_clips(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> Result<&[UiMountedCanonicalBox], crate::graph::UiGraphNodeIdentity> {
        self.surfaces
            .get(&instance.basis().semantic_surface_identity())
            .and_then(|surface| surface.occurrences.get(&instance.identity()))
            .map_or(Ok(&[]), |row| {
                row.scroll_clips.as_deref().map_err(|node| *node)
            })
    }

    pub(crate) fn surface_paint_posture(
        &self,
        instance: &crate::mounting::UiMountedInstanceIdentityView,
    ) -> super::super::UiMountedSurfacePaintPosture {
        self.surfaces
            .get(&instance.basis().semantic_surface_identity())
            .and_then(|surface| surface.occurrences.get(&instance.identity()))
            .map_or_else(Default::default, |row| row.surface_paint_posture.clone())
    }

    /// The coverage the regions inside `owner`'s Portal content leave
    /// `instance`, one of that content's occurrences. Portal content is laid
    /// out relative to its owner, so its ancestor clips include those of the
    /// owner and of every occurrence the owner is laid out in; those clip
    /// where the Portal opens from, not the content. A region laid out inside
    /// the content, such as a list scrolling inside a modal, clips it
    /// wherever the Portal presents it. Without a row for `owner` there is no
    /// telling which clips it opens from, and without resolved Scroll
    /// bindings no telling which Scroll regions hold the content, so those
    /// leave the content unclipped: its extent may exceed what the Portal
    /// shows, never fall short of it.
    pub(crate) fn portal_content_clip(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> crate::mounting::projection::UiMountedAppearanceClip {
        self.surfaces.get(&surface).map_or(
            crate::mounting::projection::UiMountedAppearanceClip::Unclipped,
            |geometry| portal_content_clip(&geometry.occurrences, owner, instance),
        )
    }
}

/// The coverage the regions inside `owner`'s Portal content leave `instance`
/// among `occurrences`, as `portal_content_clip` reads it.
fn portal_content_clip(
    occurrences: &std::collections::BTreeMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        super::UiMountedOccurrenceGeometryRow,
    >,
    owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> crate::mounting::projection::UiMountedAppearanceClip {
    use super::super::{UiMountedMosaicClipBinding, UiMountedScrollClipBinding};
    use crate::mounting::projection::UiMountedAppearanceClip;
    if !occurrences.contains_key(&owner) {
        return UiMountedAppearanceClip::Unclipped;
    }
    let Some(row) = occurrences.get(&instance) else {
        return UiMountedAppearanceClip::Unclipped;
    };
    let mut opened_from = std::collections::BTreeSet::new();
    let mut cursor = Some(owner);
    while let Some(occurrence) = cursor.filter(|occurrence| opened_from.insert(*occurrence)) {
        cursor = occurrences.get(&occurrence).and_then(|row| row.parent);
    }
    let inside = |occurrence: &worth_ui_host_contract::UiMountedInstanceIdentity| {
        !opened_from.contains(occurrence)
    };
    let mosaic = row
        .mosaic_clip_bindings
        .iter()
        .zip(row.mosaic_clips.iter())
        .filter(|(binding, _)| match binding {
            UiMountedMosaicClipBinding::Region { owner, .. } => inside(owner),
            UiMountedMosaicClipBinding::Viewport => false,
        })
        .map(|(_, clip)| *clip);
    let scroll = row
        .scroll_clip_bindings
        .as_deref()
        .ok()
        .zip(row.scroll_clips.as_deref().ok())
        .into_iter()
        .flat_map(|(bindings, clips)| bindings.iter().zip(clips))
        .filter(|(binding, _)| match binding {
            UiMountedScrollClipBinding::Region { owner, .. }
            | UiMountedScrollClipBinding::Occurrence(owner) => inside(owner),
            UiMountedScrollClipBinding::Viewport => false,
        })
        .map(|(_, clip)| *clip);
    crate::mounting::projection::ancestor_clip(mosaic.chain(scroll))
}

#[cfg(test)]
mod tests {
    use super::super::super::{
        UiMountedMosaicClipBinding as Mosaic, UiMountedScrollClipBinding as Scroll,
    };
    use super::super::UiMountedOccurrenceGeometryRow;
    use crate::mounting::projection::UiMountedAppearanceClip as Clip;
    use std::collections::BTreeMap;
    use worth_ui_host_contract::{
        UiAppearanceClip, UiMountIncarnation, UiMountedAllocationBasis, UiMountedCanonicalBox,
        UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedInstanceIdentity,
        UiMountedTransformProjection,
    };

    /// A modal opened from `owner`, a control laid out in `page`, a container
    /// whose region holds the control. Inside the modal a `list` owns a
    /// region its `row` scrolls through. The row's ancestor clips cover
    /// every kind: the viewport, the page's region, the owner's own
    /// occurrence, and the list's region as Mosaic and Scroll record it.
    struct Modal {
        owner: UiMountedInstanceIdentity,
        list: UiMountedInstanceIdentity,
        row: UiMountedInstanceIdentity,
        occurrences: BTreeMap<UiMountedInstanceIdentity, UiMountedOccurrenceGeometryRow>,
    }

    impl Modal {
        fn new() -> Self {
            let [page, owner, list, row] =
                [(); 4].map(|()| UiMountedInstanceIdentity::mint_unbound().unwrap());
            let declaration = worth_ui_dsl::UiMosaicRegionDeclarationIdentity::new(1).unwrap();
            let occurrences = [
                (page, occurrence(None, [], Ok(vec![]))),
                (
                    owner,
                    occurrence(
                        Some(page),
                        [(
                            Mosaic::Region {
                                owner: page,
                                declaration,
                            },
                            [0.0, 0.0, 800.0, 90.0],
                        )],
                        Ok(vec![]),
                    ),
                ),
                (list, occurrence(None, [], Ok(vec![]))),
                (
                    row,
                    occurrence(
                        None,
                        [
                            (Mosaic::Viewport, [0.0, 0.0, 800.0, 400.0]),
                            (
                                Mosaic::Region {
                                    owner: page,
                                    declaration,
                                },
                                [0.0, 0.0, 800.0, 90.0],
                            ),
                            (
                                Mosaic::Region {
                                    owner: list,
                                    declaration,
                                },
                                [20.0, 100.0, 300.0, 90.0],
                            ),
                        ],
                        Ok(vec![
                            (Scroll::Occurrence(owner), [0.0, 0.0, 40.0, 20.0]),
                            (
                                Scroll::Region {
                                    owner: list,
                                    declaration,
                                },
                                [20.0, 110.0, 300.0, 200.0],
                            ),
                        ]),
                    ),
                ),
            ]
            .into_iter()
            .collect();
            Self {
                owner,
                list,
                row,
                occurrences,
            }
        }

        fn clip(&self, owner: UiMountedInstanceIdentity) -> Clip {
            super::portal_content_clip(&self.occurrences, owner, self.row)
        }
    }

    #[test]
    fn a_region_inside_the_content_clips_it_wherever_the_portal_presents_it() {
        let modal = Modal::new();
        // Only the list's region counts: its Mosaic and Scroll records meet
        // in the rows the list shows.
        assert_eq!(
            modal.clip(modal.owner),
            Clip::Ancestor(UiAppearanceClip::new(20_000, 110_000, 300_000, 80_000).unwrap())
        );
    }

    #[test]
    fn clips_where_the_portal_opens_from_leave_the_content_whole() {
        let mut modal = Modal::new();
        // A Portal opened from the list, which the owner lays out, opens from
        // inside the list's region and the page's: neither clips its content.
        modal.occurrences.get_mut(&modal.list).unwrap().parent = Some(modal.owner);
        assert_eq!(modal.clip(modal.list), Clip::Unclipped);
        // Without the list's region, the row keeps only the viewport the
        // Portal is placed within, the page's region the owner sits in, and
        // the owner's own occurrence: nothing inside the content clips it.
        let row = modal.occurrences.get_mut(&modal.row).unwrap();
        row.mosaic_clip_bindings = row.mosaic_clip_bindings[..2].into();
        row.mosaic_clips = row.mosaic_clips[..2].into();
        row.scroll_clip_bindings = Ok(row.scroll_clip_bindings.as_deref().unwrap()[..1].into());
        row.scroll_clips = Ok(row.scroll_clips.as_deref().unwrap()[..1].into());
        assert_eq!(modal.clip(modal.owner), Clip::Unclipped);
    }

    #[test]
    fn content_whose_clips_cannot_be_told_apart_is_left_whole_where_unknown() {
        let mut modal = Modal::new();
        // No row for the owner: nothing tells which clips it opens from.
        let absent = UiMountedInstanceIdentity::mint_unbound().unwrap();
        assert_eq!(modal.clip(absent), Clip::Unclipped);
        // Unresolved Scroll bindings leave the Mosaic record alone to clip.
        let row = modal.occurrences.get_mut(&modal.row).unwrap();
        let unresolved = crate::graph::UiGraphNodeIdentity::new(9);
        row.scroll_clip_bindings = Err(unresolved);
        row.scroll_clips = Err(unresolved);
        assert_eq!(
            modal.clip(modal.owner),
            Clip::Ancestor(UiAppearanceClip::new(20_000, 100_000, 300_000, 90_000).unwrap())
        );
    }

    fn occurrence<const N: usize>(
        parent: Option<UiMountedInstanceIdentity>,
        mosaic: [(Mosaic, [f32; 4]); N],
        scroll: Result<Vec<(Scroll, [f32; 4])>, crate::graph::UiGraphNodeIdentity>,
    ) -> UiMountedOccurrenceGeometryRow {
        let scroll = scroll.map(|scroll| scroll.into_iter().unzip::<_, _, Vec<_>, Vec<_>>());
        UiMountedOccurrenceGeometryRow {
            parent,
            graph_node: crate::graph::UiGraphNodeIdentity::new(1),
            incarnation: UiMountIncarnation::mint_unbound().unwrap(),
            bounds: bounds([0.0, 0.0, 100.0, 80.0]),
            basis: UiMountedAllocationBasis::new(1, 1, 1, UiMountedTransformProjection::Identity),
            mosaic_clip_bindings: mosaic.map(|(binding, _)| binding).into(),
            mosaic_clips: mosaic.map(|(_, clip)| bounds(clip)).into(),
            scroll_clip_bindings: scroll.clone().map(|(bindings, _)| bindings.into()),
            scroll_clips: scroll.map(|(_, clips)| clips.into_iter().map(bounds).collect()),
            surface_paint_posture: Default::default(),
        }
    }

    fn bounds([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap()
    }
}

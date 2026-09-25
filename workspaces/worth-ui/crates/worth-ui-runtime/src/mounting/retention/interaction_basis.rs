use worth_ui_host_contract::{UiHostObservationPresentationBasis, UiMountedHitTestMechanic};

use super::UiPresentedFrameBasisRelation;
use crate::mounting::presentation::{
    UiDisplayedRect, UiDisplayedSurfaceBasis, UiPublishedMap, UiPublishedRect,
    UiPublishedToAcceptedMap, UiScrollPoseShift,
};

mod hit_rect;
pub(crate) use hit_rect::UiPresentedHitRect;

/// Exact mounting-owned evidence made available to interaction targeting,
/// read against the retained witness that displayed its binding.
pub(crate) struct UiPresentedHitTestBasis {
    displayed: UiDisplayedSurfaceBasis,
    relation: UiPresentedFrameBasisRelation,
    rows: Box<[UiPresentedHitTestRow]>,
    query_work: crate::mounting::hit_test_work::UiHitTestSpatialWork,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiPresentedHitTestRow {
    mounted: UiMountedHitTestMechanic,
    bounds: UiPresentedHitRect,
    clip_bounds: UiPresentedHitRect,
    portal_motion_target: Option<crate::runtime::motion::UiMotionTargetIdentity>,
    owns_presented_portal: bool,
}

impl UiPresentedHitTestBasis {
    pub(in crate::mounting) fn from_candidates(
        displayed: UiDisplayedSurfaceBasis,
        relation: UiPresentedFrameBasisRelation,
        query: crate::mounting::presented_hit_index::UiPresentedHitQuery,
    ) -> Self {
        Self {
            displayed,
            relation,
            rows: query.rows.into_boxed_slice(),
            query_work: query.work,
        }
    }

    pub(crate) const fn query_work(&self) -> crate::mounting::hit_test_work::UiHitTestSpatialWork {
        self.query_work
    }
    pub(crate) fn new(
        displayed: UiDisplayedSurfaceBasis,
        relation: UiPresentedFrameBasisRelation,
        rows: Box<[crate::mounting::UiMountedHitTestPresentation]>,
    ) -> Self {
        Self {
            displayed,
            relation,
            query_work: crate::mounting::hit_test_work::UiHitTestSpatialWork {
                reconstructed_rows: rows.len(),
                ..Default::default()
            },
            rows: rows
                .into_vec()
                .into_iter()
                .map(UiPresentedHitTestRow::from_mounted)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub(crate) const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.displayed.basis()
    }

    /// The retained witness basis these rows are read against.
    pub(in crate::mounting) const fn displayed(&self) -> UiDisplayedSurfaceBasis {
        self.displayed
    }

    pub(crate) const fn relation(&self) -> UiPresentedFrameBasisRelation {
        self.relation
    }

    pub(crate) fn rows(&self) -> &[UiPresentedHitTestRow] {
        &self.rows
    }

    /// Move every row by the Scroll poses committed since its frame published
    /// it, as the presented hit index moved the same row. Motion projects from
    /// committed geometry, so this follows any Motion sample already applied.
    pub(in crate::mounting) fn follow_committed_scroll_poses(
        &mut self,
        index: &crate::mounting::presented_hit_index::UiPresentedHitIndex,
    ) {
        let binding = self.displayed.binding();
        for row in &mut self.rows {
            let (translation, probes) =
                index.committed_scroll_translation(binding, row.mounted_instance());
            self.query_work.map_key_probes += probes;
            if let Some(translation) = translation {
                *row = row.scroll_translated(translation);
            }
        }
    }

    pub(crate) fn apply_motion_samples(
        &mut self,
        sampler: &crate::mounting::presentation::motion_sampling::UiMountedMotionSampler,
    ) {
        self.rows = self
            .rows
            .iter()
            .filter_map(|row| row.with_current_motion(sampler, self.displayed))
            .collect::<Vec<_>>()
            .into_boxed_slice();
    }
}

impl UiPresentedHitTestRow {
    pub(in crate::mounting) fn with_prepared_entrance(
        self,
        entrance: crate::runtime::motion::UiPreparedMotionEntrance,
    ) -> Self {
        let (Some(source), Some(initial)) = entrance.geometry() else {
            return self;
        };
        let Some(entrance) = UiPublishedMap::between(source, initial) else {
            return self;
        };
        let (bounds, clip_bounds) = self.committed();
        Self {
            bounds: UiPresentedHitRect::Published(entrance.apply(bounds)),
            clip_bounds: UiPresentedHitRect::Published(entrance.apply(clip_bounds)),
            ..self
        }
    }

    /// Where this row's frame committed it: Motion projects from the
    /// mechanic's committed boxes, whatever has since moved the row.
    fn committed(self) -> (UiPublishedRect, UiPublishedRect) {
        (
            UiPublishedRect::from_committed_box(self.mounted.bounds()),
            UiPublishedRect::from_committed_box(self.mounted.clip_bounds()),
        )
    }

    pub(in crate::mounting) fn from_mounted(
        presentation: crate::mounting::UiMountedHitTestPresentation,
    ) -> Self {
        let mounted = presentation.mechanic();
        let portal_target = presentation.portal().map(portal_motion_target);
        Self {
            bounds: UiPresentedHitRect::Published(UiPublishedRect::from_committed_box(
                mounted.bounds(),
            )),
            clip_bounds: UiPresentedHitRect::Published(UiPublishedRect::from_committed_box(
                mounted.clip_bounds(),
            )),
            mounted,
            portal_motion_target: portal_target,
            owns_presented_portal: presentation.owns_presented_portal(),
        }
    }

    pub(in crate::mounting) fn with_current_motion(
        self,
        sampler: &crate::mounting::presentation::motion_sampling::UiMountedMotionSampler,
        displayed: UiDisplayedSurfaceBasis,
    ) -> Option<Self> {
        self.with_current_motion_work(sampler, displayed).0
    }

    /// The row as the witness that displayed its binding shows it: moved by
    /// the on-screen sample of its Motion target, if one moves it.
    pub(in crate::mounting) fn with_current_motion_work(
        self,
        sampler: &crate::mounting::presentation::motion_sampling::UiMountedMotionSampler,
        displayed: UiDisplayedSurfaceBasis,
    ) -> (Option<Self>, usize) {
        let presentation = displayed.basis();
        let mut considered = 0;
        let sample = self.portal_motion_target.map_or_else(
            || {
                if self.owns_presented_portal {
                    return None;
                }
                let (sample, work) = sampler
                    .current_sample_for_with_work(self.mounted.mounted_instance(), presentation);
                considered = work;
                sample
            },
            |target| sampler.current_sample_for_target(target, presentation),
        );
        (self.with_motion_sample(sample, displayed), considered)
    }

    fn with_motion_sample(
        self,
        sample: Option<
            crate::mounting::presentation::motion_sampling::UiPresentationMotionSampleReceipt,
        >,
        displayed: UiDisplayedSurfaceBasis,
    ) -> Option<Self> {
        let Some(sample) = sample else {
            return Some(self);
        };
        if !sample.hit_test_visible() {
            return None;
        }
        let sampled = sample.geometry()?;
        let (bounds, clip) = self.committed();
        // The sampler answers only on-screen samples of the displayed binding,
        // so the retained witness of that binding displays each of them.
        let shown = |accepted| {
            UiPresentedHitRect::displayed(
                UiDisplayedRect::displayed(accepted, displayed)
                    .expect("an on-screen sample of the displayed binding is displayed"),
            )
        };
        let (bounds, clip_bounds) = if self.portal_motion_target.is_some() {
            // A Portal whose base has no area places nothing laid out in it.
            let portal = UiPublishedToAcceptedMap::of_sample(sample.base_geometry()?, sampled)?;
            (shown(portal.apply(bounds)), shown(portal.apply(clip)))
        } else {
            // The sampler keys an ordinary sample by this row's own mounted
            // instance. A sample in another space would be another row's
            // geometry: a broken sampler invariant no host input can cause,
            // so it stops here rather than leaving the row silently unmoved.
            assert_eq!(
                sampled.coordinate_space(),
                bounds.coordinate_space(),
                "a Motion sample and the hit geometry it moves share a coordinate space"
            );
            (shown(sampled), UiPresentedHitRect::Published(clip))
        };
        Some(Self {
            bounds,
            clip_bounds,
            ..self
        })
    }

    pub(crate) const fn mounted(self) -> UiMountedHitTestMechanic {
        self.mounted
    }

    /// Whether `other` is this row standing in the same place, whatever
    /// proved its geometry.
    pub(in crate::mounting) fn occupies_same_place(self, other: Self) -> bool {
        self.mounted == other.mounted
            && self.portal_motion_target == other.portal_motion_target
            && self.owns_presented_portal == other.owns_presented_portal
            && self.bounds.occupies_same_rect(other.bounds)
            && self.clip_bounds.occupies_same_rect(other.clip_bounds)
    }

    /// The same row, displaced by the distance a settled scroll pose moved the
    /// occurrence it stands for.
    ///
    /// Bounds and clip travel together. A presented hit row's clip is its own
    /// allocation, narrowed by whatever hit inset the component declares, so
    /// it belongs to the row rather than to anything the row sits inside;
    /// leaving it behind would strand the row against a clip its bounds had
    /// already left and make the occurrence unreachable everywhere.
    pub(in crate::mounting) fn scroll_translated(self, shift: UiScrollPoseShift) -> Self {
        Self {
            bounds: self.bounds.following_pose(shift),
            clip_bounds: self.clip_bounds.following_pose(shift),
            ..self
        }
    }

    pub(in crate::mounting) const fn portal_motion_target(
        self,
    ) -> Option<crate::runtime::motion::UiMotionTargetIdentity> {
        self.portal_motion_target
    }

    pub(in crate::mounting) const fn owns_presented_portal(self) -> bool {
        self.owns_presented_portal
    }

    pub(in crate::mounting) fn reattributed(
        mut self,
        receipts: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> (Self, usize) {
        let (mounted, probes) = crate::mounting::projection::reattribute_hit_test_with_probes(
            self.mounted,
            receipts.frame(),
            receipts,
        )
        .expect("indexed presented row retains exact mounted receipt membership");
        self.mounted = mounted;
        (self, probes)
    }
    pub(crate) const fn bounds(self) -> UiPresentedHitRect {
        self.bounds
    }
    pub(crate) const fn clip_bounds(self) -> UiPresentedHitRect {
        self.clip_bounds
    }
    pub(crate) const fn order(self) -> worth_ui_host_contract::UiMountedHitTestOrder {
        self.mounted.order()
    }
    pub(crate) const fn mounted_instance(
        self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted.mounted_instance()
    }
    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.mounted.node_receipt()
    }
}

fn portal_motion_target(
    portal: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> crate::runtime::motion::UiMotionTargetIdentity {
    crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
        portal.surface(),
        portal.owner(),
        portal.portal_identity(),
    )
}

#[cfg(test)]
pub(crate) fn motion_sampling_hit_test_mechanic_for_test(
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    target: crate::runtime::motion::UiMotionTargetIdentity,
    components: [f32; 4],
) -> worth_ui_host_contract::UiMountedHitTestMechanic {
    let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: components[0],
            y: components[1],
            width: components[2],
            height: components[3],
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("test hit-test geometry is canonical");
    let receipt =
        worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(presentation.frame())
            .expect("test frame identity is non-zero")
            .receipt_for(target.mounted_instance());
    worth_ui_host_contract::UiMountedHitTestMechanic::complete_from_runtime_mounting(
        worth_ui_host_contract::UiMountedHitTestCompletionInput {
            frame: presentation.frame(),
            surface: target.semantic_surface(),
            binding: presentation.binding(),
            mounted_instance: target.mounted_instance(),
            node_receipt: receipt,
            bounds,
            clip_bounds: bounds,
            order: worth_ui_host_contract::UiMountedHitTestOrder::from_runtime_plan(1),
        },
    )
    .expect("test hit-test mechanic is coherent")
}

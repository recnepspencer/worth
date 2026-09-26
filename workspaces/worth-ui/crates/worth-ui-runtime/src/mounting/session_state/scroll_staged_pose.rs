//! Moving mounted geometry staged past the frame the host shows.
//!
//! Direct input and a settle carried into a staged layout both pose the
//! geometry the next frame publishes. Each pose moves its owner's
//! descendants from where they are staged, and marks what it moved so no
//! frame prepared before it can publish the geometry it replaced.

use crate::mounting::UiMountedOccurrenceGeometryDenial as Denial;
use crate::runtime::scroll::UiScrollOffset;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

impl super::WorthUiMountedSessionState {
    /// Pose each owner `poses` names at its offset over the geometry staged
    /// on its surface, and mark what moved, or `unmoved` when the poses move
    /// nothing, with every row a settle carried there. Nothing moves while a
    /// presentation attempt is in flight.
    pub(super) fn stage_scroll_poses(
        &mut self,
        poses: impl IntoIterator<
            Item = (
                UiSemanticSurfaceIdentity,
                UiMountedInstanceIdentity,
                UiScrollOffset,
            ),
        >,
        unmoved: impl IntoIterator<Item = UiMountedInstanceIdentity>,
    ) -> Result<(), Denial> {
        if self.has_active_presentation_attempt() {
            return Err(Denial::PresentationInFlight);
        }
        let mut surfaces = std::collections::BTreeMap::<_, Vec<_>>::new();
        for (surface, owner, offset) in poses {
            surfaces.entry(surface).or_default().push((owner, offset));
        }
        let poses = surfaces
            .into_iter()
            .map(|(surface, poses)| {
                self.occurrence_geometry
                    .prepare_scroll_pose(surface, &poses, &[])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut changed = poses
            .iter()
            .flat_map(|pose| pose.changed_instances().into_vec())
            .collect::<Vec<_>>();
        if changed.is_empty() {
            changed.extend(unmoved);
        }
        for pose in &poses {
            changed.extend(self.occurrence_geometry.sample_carried(pose.surface()));
        }
        changed.sort_unstable();
        changed.dedup();
        self.identity
            .mark_occurrence_geometry_changed(&changed)
            .map_err(|_| Denial::StateRevisionExhausted)?;
        for pose in poses {
            self.occurrence_geometry
                .release_sample_carried(pose.surface());
            self.occurrence_geometry.apply_scroll_pose(pose);
        }
        Ok(())
    }

    /// Pose `owner` on `surface` at `offset` in the layout staged there, where
    /// a settle that arrived before the layout was shown left its content.
    pub(crate) fn settle_staged_layout_pose(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        owner: UiMountedInstanceIdentity,
        offset: UiScrollOffset,
    ) -> Result<(), Denial> {
        self.stage_scroll_poses([(surface, owner, offset)], [])
    }
}

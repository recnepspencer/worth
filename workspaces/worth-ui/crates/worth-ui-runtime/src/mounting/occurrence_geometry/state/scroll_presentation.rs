//! Capture exact moving membership and stationary/moving clip provenance at
//! frame preparation, while the layout owner still has that distinction.
use super::*;
use crate::mounting::presentation::work_producer::{
    UiMountedScrollMotionClip, UiMountedScrollMotionMember,
};

impl UiMountedOccurrenceGeometryState {
    pub(crate) fn scroll_presentation_members(
        &self,
        surface: UiSemanticSurfaceIdentity,
        owner: UiMountedInstanceIdentity,
    ) -> Option<std::sync::Arc<[UiMountedScrollMotionMember]>> {
        let geometry = self.surfaces.get(&surface)?;
        let descendants = geometry.scroll_index.descendants(owner);
        let mut members = Vec::with_capacity(descendants.len());
        for instance in descendants {
            let row = geometry.occurrences.get(instance)?;
            let mut clips = vec![UiMountedScrollMotionClip {
                bounds: geometry.viewport,
                owner: None,
            }];
            for (binding, bounds) in row.mosaic_clip_bindings.iter().zip(row.mosaic_clips.iter()) {
                let owner = match binding {
                    super::super::UiMountedMosaicClipBinding::Viewport => None,
                    super::super::UiMountedMosaicClipBinding::Region { owner, .. } => Some(*owner),
                };
                clips.push(UiMountedScrollMotionClip {
                    bounds: *bounds,
                    owner,
                });
            }
            for (binding, bounds) in row
                .scroll_clip_bindings
                .as_ref()
                .ok()?
                .iter()
                .zip(row.scroll_clips.as_ref().ok()?.iter())
            {
                let owner = match binding {
                    super::super::UiMountedScrollClipBinding::Viewport => None,
                    super::super::UiMountedScrollClipBinding::Occurrence(owner)
                    | super::super::UiMountedScrollClipBinding::Region { owner, .. } => {
                        Some(*owner)
                    }
                };
                clips.push(UiMountedScrollMotionClip {
                    bounds: *bounds,
                    owner,
                });
            }
            members.push(UiMountedScrollMotionMember {
                instance: *instance,
                clips: clips
                    .into_iter()
                    .map(crate::mounting::UiLaidOut::from_layout)
                    .collect(),
            });
        }
        // Layout visits members in its own order; groups find them by
        // instance.
        members.sort_unstable_by_key(|member| member.instance);
        Some(members.into())
    }
}

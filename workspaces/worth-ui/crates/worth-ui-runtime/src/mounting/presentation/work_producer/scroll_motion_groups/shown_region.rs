//! Where the bound frame shows a Scroll group. Its region stands where the
//! frame presents the region, and each member where the frame presents that
//! member: a member presented through a Portal the region is not presented
//! through is clipped by the region's viewport moved with it, as the frame
//! clips that member's paint by its moved ancestors.
use super::{UiMountedScrollMotionClip, UiMountedScrollMotionGroupInput};
use crate::mounting::{UiLaidOut, UiMountedPlacement, UiMountedScrollRegionBoxes};
use std::sync::Arc;
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedInstanceIdentity};

#[derive(Clone)]
pub(super) struct UiShownScrollGroup {
    /// The region where the frame presents it, in the client viewport: what
    /// its thumbs travel within.
    region: UiMountedScrollRegionBoxes,
    /// The region's viewport where the frame shows each member it presents
    /// elsewhere than the region, ordered by member.
    elsewhere: Arc<[(UiMountedInstanceIdentity, UiMountedCanonicalBox)]>,
}

impl UiShownScrollGroup {
    pub(super) fn new(
        region: UiMountedScrollRegionBoxes,
        mut elsewhere: Vec<(UiMountedInstanceIdentity, UiMountedCanonicalBox)>,
    ) -> Self {
        elsewhere.sort_unstable_by_key(|(member, _)| *member);
        Self {
            region,
            elsewhere: elsewhere.into(),
        }
    }

    pub(super) fn region(&self) -> UiMountedScrollRegionBoxes {
        self.region
    }

    /// The group's viewport where the frame shows `instance`'s paint: what
    /// the group clips that paint to.
    pub(super) fn viewport_showing(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> UiMountedCanonicalBox {
        self.elsewhere
            .binary_search_by_key(&instance, |(member, _)| *member)
            .map_or(self.region.viewport(), |at| self.elsewhere[at].1)
    }

    pub(super) fn elsewhere_len(&self) -> usize {
        self.elsewhere.len()
    }
}

/// The group `input` binds, shown where `frame` presents it, with each
/// member's clips where the frame presents that member. A member the frame
/// cannot show refuses the whole group, as its region would.
pub(super) fn show_group(
    frame: &crate::mounting::UiPreparedMountedFrame,
    input: &UiMountedScrollMotionGroupInput,
) -> Result<(UiShownScrollGroup, Vec<Vec<UiMountedScrollMotionClip>>), ()> {
    let show = |instance| shown_where(frame, input, instance);
    let owner = show(input.owner);
    let region = UiMountedScrollRegionBoxes::new(
        owner(input.region.map(|region| region.content()))?,
        owner(input.region.map(|region| region.viewport()))?,
    );
    let mut elsewhere = Vec::new();
    let mut member_clips = Vec::with_capacity(input.members.len());
    for member in input.members.iter() {
        let member_shown = show(member.instance);
        let viewport = member_shown(input.region.map(|region| region.viewport()))?;
        if viewport != region.viewport() {
            elsewhere.push((member.instance, viewport));
        }
        let mut clips = member
            .clips
            .iter()
            .map(|clip| {
                Ok(UiMountedScrollMotionClip {
                    bounds: member_shown(clip.map(|clip| clip.bounds))?,
                    owner: clip.in_layout_space().owner,
                })
            })
            .collect::<Result<Vec<_>, ()>>()?;
        clips.push(UiMountedScrollMotionClip {
            bounds: viewport,
            owner: Some(input.owner),
        });
        member_clips.push(clips);
    }
    Ok((UiShownScrollGroup::new(region, elsewhere), member_clips))
}

/// Shows laid-out bounds where `frame` presents `instance`, in the client
/// viewport: through a Portal when `instance` is Portal content. Content the
/// frame presents nowhere paints nothing, but stands where it is laid out so
/// a settle under way still lands its offset.
fn shown_where<'a>(
    frame: &'a crate::mounting::UiPreparedMountedFrame,
    input: &'a UiMountedScrollMotionGroupInput,
    instance: UiMountedInstanceIdentity,
) -> impl Fn(UiLaidOut<UiMountedCanonicalBox>) -> Result<UiMountedCanonicalBox, ()> + 'a {
    let placement = frame.semantic_projection().region_placement(instance);
    move |bounds| {
        let shown = match placement {
            UiMountedPlacement::Hidden => bounds.into_layout_space(),
            placement => placement
                .present(bounds)
                .ok()
                .flatten()
                .ok_or(())?
                .into_shown(),
        };
        frame
            .presentation_delta_source()
            .frame()
            .scroll_sample_viewport_bounds(input.target.semantic_surface(), shown)
            .map_err(|_| ())
    }
}

#[cfg(test)]
#[path = "shown_region_tests.rs"]
mod tests;

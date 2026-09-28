use crate::mounting::presentation::work_producer::UiMountedScrollMotionGroupInput;
use crate::runtime::{motion::UiMotionTargetIdentity, scroll::UiScrollOwnerIdentity};

impl crate::facade::WorthUiActiveApplicationSession {
    pub(super) fn prepare_scroll_motion_groups(
        &self,
    ) -> Result<Vec<UiMountedScrollMotionGroupInput>, ()> {
        let Some(scroll) = self.scroll.as_ref() else {
            return Ok(Vec::new());
        };
        let mut groups = Vec::new();
        for target in scroll.ownership_instances() {
            let Ok(chain) = scroll.ownership_chain(target) else {
                continue;
            };
            for (slot, owner) in chain.owners().iter().copied().enumerate() {
                if !matches!(owner, UiScrollOwnerIdentity::Region { .. }) {
                    continue;
                }
                let (Some((owner_instance, region)), Some(rest)) = (
                    self.mounted.scroll_region_geometry(target, slot),
                    self.mounted.scroll_region_rest(target, slot),
                ) else {
                    continue;
                };
                let surface = owner.semantic_surface();
                let Some(scale) = self.mounted.scroll_chrome_device_scale(surface) else {
                    return Err(());
                };
                let offset = self
                    .mounted
                    .mounted_scroll_pose(target, owner_instance)
                    .unwrap_or_default();
                groups.push(UiMountedScrollMotionGroupInput {
                    target: UiMotionTargetIdentity::from_scroll_region_owner(
                        surface,
                        target,
                        super::super::scroll_transition_preparation::scroll_motion_owner_key(owner),
                    ),
                    owner: owner_instance,
                    region,
                    rest,
                    offset,
                    scale,
                    chrome: self.admitted_scroll_chrome(owner).ok(),
                    members: self
                        .mounted
                        .scroll_presentation_members(surface, owner_instance)
                        .ok_or(())?,
                });
            }
        }
        Ok(groups)
    }
}

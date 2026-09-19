use super::UiMountedSemanticProjection;
use worth_ui_host_contract::UiMountedInstanceIdentity;

/// Position changes can alter the issued Portal order. Preserve both old and
/// new participants, but unchanged neighbors do not become dirty merely because
/// another overlay changed.
pub(in crate::mounting::projection) fn changed_owners(
    predecessor: Option<&super::super::UiMountedProjectionFrame>,
    successor: &[crate::mounting::UiMountedPortalOverlayProjectionInput],
) -> Vec<UiMountedInstanceIdentity> {
    let predecessor = predecessor
        .map(super::super::UiMountedProjectionFrame::portal_overlay_inputs)
        .unwrap_or(&[]);
    let mut changed = Vec::new();
    for (index, old) in predecessor.iter().enumerate() {
        if successor
            .get(index)
            .is_none_or(|new| !old.same_mounted_projection_meaning(*new))
        {
            changed.push(old.owner());
        }
    }
    for (index, new) in successor.iter().enumerate() {
        if predecessor
            .get(index)
            .is_none_or(|old| !old.same_mounted_projection_meaning(*new))
        {
            changed.push(new.owner());
        }
    }
    changed.sort_unstable();
    changed.dedup();
    changed
}

pub(in crate::mounting::projection) fn children_in_transition(
    predecessor: Option<&UiMountedSemanticProjection>,
    successor: &UiMountedSemanticProjection,
    owners: &[UiMountedInstanceIdentity],
) -> (Vec<UiMountedInstanceIdentity>, usize) {
    let (mut selected, mut work) = successor.portal_children_for_owners(owners);
    if let Some(previous) = predecessor {
        let (retired, previous_work) = previous.portal_children_for_owners(owners);
        selected.extend(retired);
        work = work
            .checked_add(previous_work)
            .expect("Portal selection work fits address space");
    }
    selected.sort_unstable();
    selected.dedup();
    (selected, work)
}

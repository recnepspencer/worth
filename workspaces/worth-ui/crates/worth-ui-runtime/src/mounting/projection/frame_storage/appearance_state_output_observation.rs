use super::{UiMountedAppearanceStateMembers, UiMountedAppearanceStateMembership};

impl UiMountedAppearanceStateMembers {
    pub(in crate::mounting::projection::frame_storage) fn pending_attempts(
        &self,
    ) -> Vec<crate::runtime::appearance::UiAppearanceProjectionAttempt> {
        self.primary
            .iter()
            .filter_map(|(_, membership)| match membership {
                UiMountedAppearanceStateMembership::Staged { attempt, .. } => Some(attempt.clone()),
                _ => None,
            })
            .collect()
    }

    pub(in crate::mounting::projection::frame_storage) fn physical_node_receipts(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedNodeReceiptIdentity> {
        self.primary
            .iter()
            .filter_map(|(_, membership)| match membership {
                UiMountedAppearanceStateMembership::Retained(entry) => {
                    entry.sidecar.current_node_receipt()
                }
                UiMountedAppearanceStateMembership::PhysicalOnly(physical) => {
                    physical.sidecar.current_node_receipt()
                }
                UiMountedAppearanceStateMembership::Staged {
                    predecessor: Some(entry),
                    ..
                } => entry.sidecar().current_node_receipt(),
                _ => None,
            })
            .collect()
    }
}

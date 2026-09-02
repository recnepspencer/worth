use crate::runtime::portal::UiPortalStackSnapshot;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiOverlayPortalOwnerExport {
    snapshot: UiPortalStackSnapshot,
}

impl UiOverlayPortalOwnerExport {
    pub(super) fn from_snapshot(snapshot: UiPortalStackSnapshot) -> Self {
        Self { snapshot }
    }

    pub(super) fn from_owner(owner: &crate::runtime::portal::UiPortalRuntimeState) -> Self {
        Self {
            snapshot: owner.stack_snapshot(),
        }
    }

    pub(super) fn into_snapshot(self) -> UiPortalStackSnapshot {
        self.snapshot
    }
}

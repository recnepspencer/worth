use super::{UiMountedAppearanceFrameState, UiMountedProjectionFrame};

#[derive(Clone)]
pub(crate) struct UiMountedProjectionFrameOwner {
    pub(super) projection: std::rc::Rc<UiMountedProjectionFrame>,
    pub(super) appearance: UiMountedAppearanceFrameState,
    theme_revision: Option<u64>,
}

impl UiMountedProjectionFrameOwner {
    pub(crate) fn new(
        projection: std::rc::Rc<UiMountedProjectionFrame>,
        appearance: UiMountedAppearanceFrameState,
        theme_revision: Option<u64>,
    ) -> Self {
        Self {
            projection,
            appearance,
            theme_revision,
        }
    }

    pub(crate) fn projection(&self) -> &UiMountedProjectionFrame {
        &self.projection
    }

    pub(crate) fn projection_rc(&self) -> std::rc::Rc<UiMountedProjectionFrame> {
        std::rc::Rc::clone(&self.projection)
    }

    pub(crate) fn appearance(&self) -> &UiMountedAppearanceFrameState {
        &self.appearance
    }

    pub(crate) const fn theme_revision(&self) -> Option<u64> {
        self.theme_revision
    }

    pub(crate) fn projection_mut_unique(&mut self) -> Option<&mut UiMountedProjectionFrame> {
        std::rc::Rc::get_mut(&mut self.projection)
    }
}

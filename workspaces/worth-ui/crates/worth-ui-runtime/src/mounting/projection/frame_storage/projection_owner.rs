use super::{UiMountedAppearanceFrameState, UiMountedProjectionFrame};

#[derive(Clone)]
pub(crate) struct UiMountedProjectionFrameOwner {
    pub(in crate::mounting) application_text_publication:
        Option<std::rc::Rc<crate::runtime::presentation_state::UiApplicationTextPublication>>,
    pub(super) projection: std::rc::Rc<UiMountedProjectionFrame>,
    pub(super) appearance: UiMountedAppearanceFrameState,
    pub(in crate::mounting) pointer: super::super::UiMountedPointerAffordanceState,
    theme_revision: Option<u64>,
    pub(super) unpublished_appearance: Result<
        Option<std::rc::Rc<worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection>>,
        super::UiMountedAppearanceOutputDenial,
    >,
}

impl UiMountedProjectionFrameOwner {
    pub(crate) fn new(
        projection: std::rc::Rc<UiMountedProjectionFrame>,
        appearance: UiMountedAppearanceFrameState,
        theme_revision: Option<u64>,
        pointer: super::super::UiMountedPointerAffordanceState,
    ) -> Self {
        Self {
            application_text_publication: None,
            projection,
            appearance,
            pointer,
            theme_revision,
            unpublished_appearance: Ok(None),
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

    pub(crate) fn unpublished_appearance(
        &self,
    ) -> Result<
        Option<&worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection>,
        &super::UiMountedAppearanceOutputDenial,
    > {
        self.unpublished_appearance
            .as_ref()
            .map(|projection| projection.as_deref())
    }

    pub(crate) const fn theme_revision(&self) -> Option<u64> {
        self.theme_revision
    }

    pub(crate) fn projection_mut_unique(&mut self) -> Option<&mut UiMountedProjectionFrame> {
        std::rc::Rc::get_mut(&mut self.projection)
    }
}

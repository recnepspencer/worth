//! Hit regions and text rows a Portal presents.
//!
//! The host contract moves these rows itself, as it owns their geometry. A
//! runtime reader reaches that move only through a placement.

use super::{UiPortalMove, UiPortalPresentable};
use worth_ui_host_contract::{
    UiMountedHitTestCompletionDenial, UiMountedHitTestMechanic,
    UiMountedSemanticTextCompletionDenial, UiMountedSemanticTextMechanic,
};

impl UiPortalPresentable for UiMountedHitTestMechanic {
    type Denial = UiMountedHitTestCompletionDenial;

    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Self::Denial> {
        self.presented_within_portal(by.portal(), by.source_anchor())
    }
}

impl UiPortalPresentable for UiMountedSemanticTextMechanic {
    type Denial = UiMountedSemanticTextCompletionDenial;

    fn moved(self, by: &UiPortalMove) -> Result<Option<Self>, Self::Denial> {
        self.presented_within_portal(by.portal(), by.source_anchor())
    }
}

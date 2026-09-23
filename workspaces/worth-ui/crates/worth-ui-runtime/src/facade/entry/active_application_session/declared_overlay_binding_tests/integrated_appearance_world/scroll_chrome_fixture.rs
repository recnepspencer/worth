//! The scroll chrome a World declares, and the roles that paint it.
//!
//! Chrome is not derived for a region that does not declare it, so the
//! scenarios that press and drag a thumb need a region whose descriptor names
//! a chrome contract. The contract names two appearance roles, and chrome
//! resolves a role only when it applies to any component: a bar is painted
//! beside the region rather than on one of the components the World mounts.
//! So the two roles here carry the first component's aspects and partitions --
//! whatever the theme this World declares can resolve -- under identities of
//! their own and an applicability of their own.
//!
//! Only the block axis overflows in this World's scrollable region, so that is
//! the axis the contract declares.

use worth_ui_dsl::{
    UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity,
};

pub(super) const TRACK_ROLE: &str = "overlay.scroll-track";
pub(super) const THUMB_ROLE: &str = "overlay.scroll-thumb";

/// What a region declares when it wants bars.
pub(super) fn contract() -> crate::capability::UiScrollChromeContract {
    crate::capability::UiScrollChromeContract::new(
        crate::capability::UiScrollAxisSupport::Block,
        UiAppearanceRoleIdentity::new(TRACK_ROLE).expect("a well formed role identity"),
        UiAppearanceRoleIdentity::new(THUMB_ROLE).expect("a well formed role identity"),
    )
    .expect("a track and a thumb are two distinct roles")
}

/// One chrome role: the World's own painted aspects, under `name`, applicable
/// wherever chrome is painted.
fn painted_beside_any_component(name: &str) -> UiAppearanceRoleDeclaration {
    let region = super::authored::role(0);
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new(name).expect("a well formed role identity"),
        region.revision(),
        UiAppearanceRoleApplicability::AnyComponent,
        region.aspect_contract(),
        region.partitions().iter().cloned(),
    )
    .expect("the World's own aspects readmit under another identity")
}

/// The roles a World has to register for the chrome its region declares.
///
/// A World whose region declares no chrome registers none, so every scenario
/// that does not press a bar keeps the role registry it always had.
pub(super) fn declared_chrome_roles(
    region: &crate::capability::MosaicRegionKindDescriptor,
) -> Vec<UiAppearanceRoleDeclaration> {
    match region.scroll_chrome() {
        None => Vec::new(),
        Some(_) => vec![
            painted_beside_any_component(TRACK_ROLE),
            painted_beside_any_component(THUMB_ROLE),
        ],
    }
}

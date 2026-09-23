//! The appearance roles a theme binding has to admit for one application.
//!
//! Three declaration families paint with a role: the nodes that consume one,
//! the authored backdrops that name one, and the scroll regions whose chrome
//! contract names a track and a thumb. The theme binding admits exactly this
//! set, so every site that prepares a binding derives it here rather than
//! recounting the families itself.
use super::WorthUiApp;

impl WorthUiApp {
    /// Every role the active theme must admit, sorted and deduplicated.
    pub(in crate::facade::entry) fn required_appearance_role_identities(
        &self,
    ) -> Vec<worth_ui_dsl::UiAppearanceRoleIdentity> {
        let authority = self.prepared_authority();
        let mut roles = authority
            .consumed_fact_index()
            .appearance_required_role_identities()
            .into_vec();
        roles.extend(
            authority
                .authored_overlay_material()
                .backdrop_appearance_role_identities(),
        );
        roles.extend(
            self.capabilities()
                .mosaic_regions()
                .scroll_chrome_role_identities(),
        );
        roles.sort();
        roles.dedup();
        roles
    }
}

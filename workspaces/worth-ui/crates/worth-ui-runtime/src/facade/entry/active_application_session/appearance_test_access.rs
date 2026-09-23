//! Test-only access to the session's appearance owner snapshot and theme
//! bindings, so appearance tests can inspect and replace what a running
//! session holds without widening the production surface.

use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) const fn has_appearance_owner_snapshot_for_test(&self) -> bool {
        self.appearance_owner_snapshot.is_some()
    }

    pub(crate) const fn appearance_owner_snapshot_for_test(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.appearance_owner_snapshot.as_ref()
    }

    pub(crate) fn replace_appearance_theme_definition_for_test(
        &mut self,
        definition: &str,
        changed_token: &crate::capability::ThemeTokenId,
    ) {
        let receipts = {
            let themes = self
                .capabilities()
                .appearance_themes()
                .expect("appearance test session must carry a theme bundle");
            let identity = crate::capability::UiThemeDefinitionIdentity::new(definition).unwrap();
            let roles = self.capabilities().appearance_roles();
            self.presentation
                .appearance_theme_state()
                .expect("appearance test session must have active theme bindings")
                .active_bindings()
                .map(|binding| {
                    crate::runtime::appearance::UiThemeCapabilityAdmission::
                        from_frozen_capabilities(
                            themes,
                            &identity,
                            roles,
                            binding.capability().host_profile(),
                        )
                        .unwrap()
                        .issue(
                            binding
                                .capability()
                                .required_roles()
                                .iter()
                                .map(|role| role.identity().clone()),
                            binding.surface(),
                            self.active_generation_identity(),
                        )
                        .unwrap()
                })
                .collect::<Vec<_>>()
        };
        let authority = self.application.prepared_authority();
        let index = authority.consumed_fact_index();
        let declarations = authority.authored_declaration_lookup();
        let authored = declarations
            .theme_token_declaration_identity(changed_token.as_str())
            .unwrap_or(changed_token.as_str());
        let selected = index
            .select_appearance_slot_consumers(index.basis(), changed_token.as_str(), authored)
            .expect("test theme switch selects declared slot consumers");
        for receipt in receipts {
            let (batch, _) =
                crate::runtime::appearance::UiAppearanceInvalidationBatch::theme_surface(
                    index,
                    &self.mounted,
                    self.graph(),
                    receipt.surface(),
                    selected.consumers(),
                )
                .expect("test theme switch selects only the bound surface");
            self.presentation
                .replace_appearance_theme_binding_for_test(receipt);
            self.presentation
                .queue_appearance_invalidation(batch)
                .expect("test theme switch queues its surface invalidation");
        }
    }
}

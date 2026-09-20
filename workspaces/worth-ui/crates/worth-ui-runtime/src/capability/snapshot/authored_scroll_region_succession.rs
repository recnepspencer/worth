use super::CapabilitySnapshot;
use crate::capability::{
    CapabilitySnapshotBuilder, CapabilitySnapshotFreezeInput, UiAuthoredScrollRegionCause,
    UiAuthoredScrollRegionClauses, UiAuthoredScrollRegionDenial, UiScrollChromeContractDenial,
};

impl CapabilitySnapshot {
    /// Succeed this snapshot with the scroll clauses an authored `scroll` block
    /// states for the region kind it names.
    ///
    /// `Ok(None)` means every authored clause already agreed with its
    /// registered descriptor, so this snapshot still states the whole truth and
    /// no successor is owed.
    pub(crate) fn refreeze_authored_scroll_regions(
        &self,
        clauses: &[UiAuthoredScrollRegionClauses],
    ) -> Result<Option<Self>, UiAuthoredScrollRegionDenial> {
        self.admit_authored_chrome_roles(clauses)?;
        let Some(mosaic_regions) = self
            .mosaic_regions
            .prepare_authored_scroll_succession(clauses)?
        else {
            return Ok(None);
        };
        Ok(Some(
            CapabilitySnapshotBuilder::new(CapabilitySnapshotFreezeInput {
                registered_capabilities: self.registered_capabilities.clone(),
                appearance_roles: self.appearance_roles.clone(),
                appearance_themes: self.appearance_themes.clone(),
                commands: self.commands.clone(),
                command_projections: self.command_projections.clone(),
                components: self.components.clone(),
                icons: self.icons.clone(),
                intent_definitions: self.intent_definitions.clone(),
                surfaces: self.surfaces.clone(),
                mosaic_regions,
                mosaic_placement_policies: self.mosaic_placement_policies.clone(),
                mosaic_sizing_contracts: self.mosaic_sizing_contracts.clone(),
                mosaic_state_slots: self.mosaic_state_slots.clone(),
                native_capabilities: self.native_capabilities.clone(),
                plugin_slots: self.plugin_slots.clone(),
                view_bindings: self.view_bindings.clone(),
                runtime_outcome_projections: self.runtime_outcome_projections.clone(),
                settings: self.settings.clone(),
                task_presentations: self.task_presentations.clone(),
                theme_tokens: self.theme_tokens.clone(),
                support_catalog: self.support_catalog.clone(),
            })
            .freeze(),
        ))
    }

    /// A chrome contract names the roles that paint the track and the thumb.
    /// This is where those names meet the appearance role registry, because the
    /// contract itself holds no registry and cannot answer the question.
    fn admit_authored_chrome_roles(
        &self,
        clauses: &[UiAuthoredScrollRegionClauses],
    ) -> Result<(), UiAuthoredScrollRegionDenial> {
        for clause in clauses {
            let Some(chrome) = clause.chrome() else {
                continue;
            };
            for role in [chrome.track_role(), chrome.thumb_role()] {
                if self.appearance_roles.get(&role).is_none() {
                    return Err(clause.denied(UiAuthoredScrollRegionCause::Chrome(
                        UiScrollChromeContractDenial::UnregisteredRole,
                    )));
                }
            }
        }
        Ok(())
    }
}

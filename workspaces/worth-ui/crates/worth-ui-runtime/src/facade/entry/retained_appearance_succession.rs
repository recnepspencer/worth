use super::{UiPreparedAppearanceGenerationSuccession, WorthUiActiveApplicationSession};
use crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession;

impl WorthUiActiveApplicationSession {
    pub(super) fn commit_retained_appearance_succession(
        &mut self,
        themes: UiPreparedAppearanceGenerationSuccession,
        owners: UiPreparedRetainedAppearanceOwnerSuccession,
    ) {
        self.mounted.commit_retained_appearance_generation(&owners);
        self.commit_appearance_generation_succession(themes);
        self.appearance_owner_snapshot = owners.into_successor();
        self.refresh_appearance_owner_receipt_sources();
    }

    pub(super) fn prepare_retained_appearance_owners(
        &self,
        successor: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        themes: &UiPreparedAppearanceGenerationSuccession,
    ) -> Result<
        UiPreparedRetainedAppearanceOwnerSuccession,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let predecessor = self.application.prepared_authority();
        if !predecessor
            .consumed_fact_index()
            .has_same_appearance_consumer_contract(successor.consumed_fact_index())
            || predecessor.capabilities().appearance_roles()
                != successor.capabilities().appearance_roles()
            || predecessor.capabilities().appearance_themes()
                != successor.capabilities().appearance_themes()
            || predecessor.capabilities().theme_tokens() != successor.capabilities().theme_tokens()
            || (predecessor.consumed_fact_index().has_appearance_consumers()
                && self.appearance_owner_snapshot.is_none())
            || self
                .appearance_owner_snapshot
                .as_ref()
                .is_some_and(|snapshot| {
                    snapshot.demand() != successor.consumed_fact_index().appearance_axis_demand()
                })
        {
            return Err(
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation,
            );
        }
        self.capture_retained_appearance_owners(themes, successor.capabilities().digest().as_u64())
            .ok_or(crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation)
    }

    pub(super) fn capture_retained_appearance_owners(
        &self,
        themes: &UiPreparedAppearanceGenerationSuccession,
        successor_source_basis: u64,
    ) -> Option<UiPreparedRetainedAppearanceOwnerSuccession> {
        let owners = UiPreparedRetainedAppearanceOwnerSuccession::prepare(
            self.appearance_owner_snapshot.as_ref(),
            themes.theme(),
            self.focus.as_ref(),
            self.selection.as_ref(),
            &self.intent_admission,
            &self.intent_application_facts,
            &self.interaction,
            successor_source_basis,
        )?;
        self.mounted
            .admits_retained_appearance_generation(&owners)
            .then_some(owners)
    }
}

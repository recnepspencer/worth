use super::WorthUiActiveApplicationSession;
use crate::runtime::appearance::{
    UiThemeCapabilityReceiptDenial, UiThemeSwitchDenial,
    UiThemeSwitchOriginAdmissionDenial as OriginDenial,
};

mod programmatic;
pub use programmatic::UiProgrammaticThemeSwitchPreparationDenial;

#[derive(Debug, PartialEq)]
pub enum UiThemeSwitchPreparationDenial {
    Origin(OriginDenial),
    Admission(UiThemeSwitchDenial),
    Capability(UiThemeCapabilityReceiptDenial),
    Selection(crate::runtime::appearance::UiThemeSwitchSelectionDenial),
}

impl WorthUiActiveApplicationSession {
    /// Continue a product action only after its admitted posture reached publication.
    pub fn issue_theme_switch_origin_from_publication(
        &self,
        receipt: &crate::runtime::rebind::UiRebindReceipt,
    ) -> Result<crate::runtime::appearance::UiThemeSwitchOrigin, OriginDenial> {
        crate::runtime::appearance::UiThemeSwitchOrigin::admit_published_intent(self, receipt)
    }

    pub fn active_theme_binding(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&crate::runtime::appearance::UiActiveThemeBinding> {
        self.presentation.active_appearance_theme_binding(surface)
    }

    pub fn admit_appearance_theme(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        definition: &crate::capability::UiThemeDefinitionIdentity,
    ) -> Result<crate::runtime::appearance::UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial>
    {
        let capabilities = self.application.capabilities();
        let current = self
            .active_theme_binding(surface)
            .ok_or(UiThemeCapabilityReceiptDenial::StaleBinding)?;
        let profile = self
            .host_session
            .capability_report()
            .appearance_profile()
            .ok_or(UiThemeCapabilityReceiptDenial::MissingHostProfile)?;
        crate::runtime::appearance::UiThemeCapabilityAdmission::from_frozen_capabilities(
            capabilities
                .appearance_themes()
                .ok_or(UiThemeCapabilityReceiptDenial::MissingBundle)?,
            definition,
            capabilities.appearance_roles(),
            profile,
        )?
        .issue(
            current
                .capability()
                .required_roles()
                .iter()
                .map(|role| role.identity().clone()),
            surface,
            self.active_generation_identity(),
        )
    }

    pub fn prepare_theme_switch(
        &mut self,
        request: crate::runtime::appearance::UiThemeSwitchRequest,
    ) -> Result<
        crate::runtime::observation::UiChangeClassificationOutcome,
        UiThemeSwitchPreparationDenial,
    > {
        let origin = request.origin();
        let deny = |denial| UiThemeSwitchPreparationDenial::Origin(denial);
        if origin.session() != self.session_identity() {
            return Err(deny(OriginDenial::ForeignSession));
        }
        if origin.source_basis() != self.capabilities().digest().as_u64() {
            return Err(deny(OriginDenial::StaleSourceBasis));
        }
        if origin.generation() != &self.active_generation_identity() {
            return Err(deny(OriginDenial::StaleApplicationGeneration));
        }
        let owners = self
            .appearance_owner_snapshot
            .as_ref()
            .ok_or_else(|| deny(OriginDenial::MissingAppearanceGeneration))?;
        if owners.turn() != origin.turn() {
            return Err(deny(OriginDenial::ObservationNotClosed));
        }
        let basis = crate::runtime::observation::UiChangeClassificationBasis::new(
            origin.session(),
            origin.source_basis(),
            origin.turn(),
            origin.observation_count(),
            self.generation_identity().clone(),
        );
        if request.capability().surface() != request.surface() {
            return Err(UiThemeSwitchPreparationDenial::Admission(
                UiThemeSwitchDenial::WrongSurfaceCapability,
            ));
        }
        let binding = self.active_theme_binding(request.surface()).ok_or(
            UiThemeSwitchPreparationDenial::Admission(UiThemeSwitchDenial::MissingActiveBinding),
        )?;
        if binding.binding_generation() != request.expected_binding_generation() {
            return Err(UiThemeSwitchPreparationDenial::Admission(
                UiThemeSwitchDenial::StaleBinding,
            ));
        }
        if binding.capability() == request.capability() {
            self.presentation
                .settle_unchanged_appearance_theme_switch(request)
                .map_err(UiThemeSwitchPreparationDenial::Admission)?;
            return Ok(
                crate::runtime::observation::UiChangeClassificationOutcome::ObservedNoChange(
                    crate::runtime::observation::UiObservedNoChangeReceipt::new(basis),
                ),
            );
        }
        let predecessor = binding.clone();
        let prepared = self
            .presentation
            .prepare_appearance_theme_switch(request)
            .map_err(UiThemeSwitchPreparationDenial::Admission)?;
        let change = crate::runtime::appearance::UiThemeSwitchChange::prepare(
            prepared,
            &predecessor,
            self.application.prepared_authority(),
            &self.mounted,
            self.graph(),
        )
        .map_err(UiThemeSwitchPreparationDenial::Selection)?;
        Ok(
            crate::runtime::observation::UiChangeClassificationOutcome::Changed(
                crate::runtime::observation::UiClassifiedChange::from_theme_switch(basis, change),
            ),
        )
    }
}

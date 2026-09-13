use crate::runtime::appearance::{
    UiThemeCapabilityReceipt, UiThemeSwitchOrigin, UiThemeSwitchRequest,
};
use crate::runtime::observation::*;

#[derive(Debug)]
pub enum UiProgrammaticThemeSwitchPreparationDenial {
    ManagedRebindAlreadyInFlight,
    ObservationTurn(UiObservationTurnDenial),
    ObservationClose(UiObservationAdmissionDenial),
    Origin(crate::runtime::appearance::UiThemeSwitchOriginAdmissionDenial),
    Classification(UiChangeClassificationDenial),
    UnexpectedClassification,
}

impl super::WorthUiActiveApplicationSession {
    /// Observe current owners for an explicit programmatic request. This grants
    /// request provenance only; theme admission, CAS and publication follow in
    /// the existing rebind lifecycle.
    pub fn prepare_programmatic_theme_switch(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        expected_binding_generation: u64,
        capability: UiThemeCapabilityReceipt,
    ) -> Result<UiThemeSwitchRequest, UiProgrammaticThemeSwitchPreparationDenial> {
        use UiProgrammaticThemeSwitchPreparationDenial as Denial;
        let admitted = self
            .begin_observation_turn()
            .map_err(Denial::ObservationTurn)?
            .seal()
            .map_err(Denial::ObservationClose)?;
        let origin =
            UiThemeSwitchOrigin::admit_programmatic(self, &admitted).map_err(Denial::Origin)?;
        match self
            .classify_observations(admitted)
            .map_err(Denial::Classification)?
        {
            UiChangeClassificationOutcome::ObservedNoChange(_) => {}
            UiChangeClassificationOutcome::Changed(_)
            | UiChangeClassificationOutcome::EvidenceOnly(_) => {
                return Err(Denial::UnexpectedClassification)
            }
        }
        Ok(UiThemeSwitchRequest::new(
            origin,
            surface,
            expected_binding_generation,
            capability,
        ))
    }
}

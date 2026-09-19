#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum UiThemeSwitchOriginFamily {
    SourceEditObservation,
    ProgrammaticObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiThemeSwitchOriginAdmissionDenial {
    MissingRequiredObservationFamily,
    ForeignSession,
    StaleSourceBasis,
    MissingAppearanceGeneration,
    StaleApplicationGeneration,
    ObservationNotClosed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiThemeSwitchOrigin {
    family: UiThemeSwitchOriginFamily,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    turn: crate::runtime::observation::UiObservationTurnIdentity,
    source_basis: u64,
    observation_count: usize,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiThemeSwitchRequest {
    pub(super) origin: UiThemeSwitchOrigin,
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) expected_binding_generation: u64,
    pub(super) capability: super::UiThemeCapabilityReceipt,
}

impl UiThemeSwitchOrigin {
    pub(crate) fn admit_published_intent(
        owner: &crate::facade::WorthUiActiveApplicationSession,
        receipt: &crate::runtime::rebind::UiRebindReceipt,
    ) -> Result<Self, UiThemeSwitchOriginAdmissionDenial> {
        let plan = receipt.plan();
        let basis = plan.basis().classification();
        if basis.session() != owner.session_identity() {
            return Err(UiThemeSwitchOriginAdmissionDenial::ForeignSession);
        }
        if basis.source_basis() != owner.capabilities().digest().as_u64() {
            return Err(UiThemeSwitchOriginAdmissionDenial::StaleSourceBasis);
        }
        if receipt.active_generation() != owner.generation_identity() {
            return Err(UiThemeSwitchOriginAdmissionDenial::StaleApplicationGeneration);
        }
        // Publication consumed the executable semantic proof. Its retained
        // classification and source lineage identify the accepted action.
        if plan.source_candidate_artifact_digest().is_some()
            || !plan.scope().is_some_and(|scope| {
                scope.facts().iter().any(|fact| {
                    fact.family() == crate::fact_contract::UiProducedFactFamily::IntentPosture
                })
            })
            || receipt.mounted_publication().is_none()
        {
            return Err(UiThemeSwitchOriginAdmissionDenial::MissingRequiredObservationFamily);
        }
        Ok(Self {
            family: UiThemeSwitchOriginFamily::ProgrammaticObservation,
            session: basis.session(),
            turn: basis.turn(),
            source_basis: basis.source_basis(),
            observation_count: basis.observation_count(),
            generation: owner.active_generation_identity(),
        })
    }

    pub(crate) fn admit_current(
        owner: &crate::facade::WorthUiActiveApplicationSession,
        admitted: &crate::runtime::observation::UiAdmittedObservationSet,
        family: UiThemeSwitchOriginFamily,
    ) -> Result<Self, UiThemeSwitchOriginAdmissionDenial> {
        let origin = Self::from_closed_observation(owner, admitted, family)?;
        let required = match family {
            UiThemeSwitchOriginFamily::SourceEditObservation => {
                crate::runtime::observation::UiObservationFamily::AuthoredSource
            }
            UiThemeSwitchOriginFamily::ProgrammaticObservation => {
                crate::runtime::observation::UiObservationFamily::IntentPosture
            }
        };
        if !admitted
            .observations()
            .iter()
            .any(|observation| observation.family() == required)
        {
            return Err(UiThemeSwitchOriginAdmissionDenial::MissingRequiredObservationFamily);
        }
        Ok(origin)
    }

    pub(crate) fn admit_programmatic(
        owner: &crate::facade::WorthUiActiveApplicationSession,
        admitted: &crate::runtime::observation::UiAdmittedObservationSet,
    ) -> Result<Self, UiThemeSwitchOriginAdmissionDenial> {
        // Programmatic input closes current owners; it does not fabricate a
        // source edit or an interaction result to authorize a theme request.
        if !admitted.observations().is_empty() {
            return Err(UiThemeSwitchOriginAdmissionDenial::MissingRequiredObservationFamily);
        }
        Self::from_closed_observation(
            owner,
            admitted,
            UiThemeSwitchOriginFamily::ProgrammaticObservation,
        )
    }

    fn from_closed_observation(
        owner: &crate::facade::WorthUiActiveApplicationSession,
        admitted: &crate::runtime::observation::UiAdmittedObservationSet,
        family: UiThemeSwitchOriginFamily,
    ) -> Result<Self, UiThemeSwitchOriginAdmissionDenial> {
        if admitted.session() != owner.session_identity() {
            return Err(UiThemeSwitchOriginAdmissionDenial::ForeignSession);
        }
        if admitted.source_basis() != owner.capabilities().digest().as_u64() {
            return Err(UiThemeSwitchOriginAdmissionDenial::StaleSourceBasis);
        }
        let carried_generation = admitted
            .appearance_owner_snapshot()
            .map(crate::runtime::appearance::UiAppearanceOwnerSnapshot::generation)
            .ok_or(UiThemeSwitchOriginAdmissionDenial::MissingAppearanceGeneration)?;
        if carried_generation != &owner.active_generation_identity() {
            return Err(UiThemeSwitchOriginAdmissionDenial::StaleApplicationGeneration);
        }
        Ok(Self {
            family,
            session: admitted.session(),
            turn: admitted.turn(),
            source_basis: admitted.source_basis(),
            observation_count: admitted.observations().len(),
            generation: carried_generation.clone(),
        })
    }

    pub(crate) const fn observation_count(&self) -> usize {
        self.observation_count
    }

    pub(crate) const fn family(&self) -> UiThemeSwitchOriginFamily {
        self.family
    }
    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }
    pub(crate) const fn turn(&self) -> crate::runtime::observation::UiObservationTurnIdentity {
        self.turn
    }
    pub(crate) const fn source_basis(&self) -> u64 {
        self.source_basis
    }
    pub(crate) const fn generation(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }
}

impl UiThemeSwitchRequest {
    pub(crate) fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) fn origin(&self) -> &UiThemeSwitchOrigin {
        &self.origin
    }
    pub(crate) fn capability(&self) -> &super::UiThemeCapabilityReceipt {
        &self.capability
    }
    pub(crate) fn expected_binding_generation(&self) -> u64 {
        self.expected_binding_generation
    }
    pub fn new(
        origin: UiThemeSwitchOrigin,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        expected_binding_generation: u64,
        capability: super::UiThemeCapabilityReceipt,
    ) -> Self {
        Self {
            origin,
            surface,
            expected_binding_generation,
            capability,
        }
    }
}

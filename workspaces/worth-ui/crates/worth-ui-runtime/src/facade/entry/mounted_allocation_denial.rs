use worth_ui_host_contract::UiMeasurementEvidenceFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiMountedAllocationRuntimeStage {
    CatalogPreparation,
}

#[derive(Debug)]
pub enum WorthUiMountedAllocationEstablishmentDenial {
    PresentationInFlight,
    NoAllocationPlanningNodes,
    MissingMountedInstance(crate::graph::UiGraphNodeIdentity),
    MountedIdentity(crate::mounting::UiMountedIdentityDenial),
    GraphMountEligibility(crate::graph::UiGraphMountEligibilityAdmissionDenial),
    CandidateTouch(crate::obligations::touch::UiGraphTouchDenial),
    MissingCandidateMountTransition(crate::graph::UiGraphNodeIdentity),
    CandidateAllocationPartition,
    StaleGraphSuccessor,
    MissingMeasurementRequest(UiMeasurementEvidenceFamily),
    MeasurementRequestIdentityExhausted,
    HostMeasurement(crate::host::UiHostMeasurementEvidenceDenial),
    MeasurementBasis {
        node: crate::graph::UiGraphNodeIdentity,
        denial: crate::evidence::UiMeasurementBasisDenial,
    },
    CatalogAdmission(crate::graph::UiAllocationCatalogBasisAdmissionDenial),
    CandidateCatalogAdmission(crate::graph::UiAllocationCatalogDeltaAdmissionDenial),
    AppearanceTheme(crate::runtime::appearance::UiThemeCapabilityReceiptDenial),
    AppearanceThemeSuccession(
        crate::runtime::presentation_state::UiAppearanceGenerationSuccessionDenial,
    ),
    AppearanceInspectionSuccession(
        crate::runtime::appearance::UiAppearanceInspectionGenerationSuccessionDenial,
    ),
    Activation(crate::runtime::WorthUiAllocationCatalogActivationDenial),
    Runtime(WorthUiMountedAllocationRuntimeStage),
}

pub(super) fn map_initial_activation_denial(
    denial: crate::runtime::WorthUiInitialMountedAllocationActivationDenial,
) -> WorthUiMountedAllocationEstablishmentDenial {
    WorthUiMountedAllocationEstablishmentDenial::Activation(denial.into_public_denial())
}

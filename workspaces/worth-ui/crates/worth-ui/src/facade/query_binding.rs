//! Product-facing Query view and projection contracts.
//!
//! Query execution, native access, and patch translation remain owned by
//! `worth-ui-query-binding`. The product facade exposes proof-carrying view and
//! projection declaration, observation, installation, and source-lifecycle
//! contracts without exposing the raw Query runtime.

pub use worth_ui_runtime::facade::entry::{
    WorthUiProjectionRegistrationError, WorthUiQueryViewRegistrationError,
};
pub use worth_ui_runtime::facade::query_binding::{
    UiApplicationScalarProjectionObservation, UiApplicationScalarProjectionRegistration,
    UiCollectionCompleteness, UiCollectionContinuation, UiCollectionProjectionBinding,
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionObservation,
    UiCollectionProjectionRegistration, UiCollectionProjectionRowReference,
    UiCollectionProjectionTextRow, UiCollectionProjectionValue, UiCollectionSchemaRequirement,
    UiInstalledProjectionView, UiNativeTextValue, UiPresentProjection, UiProjectionAvailability,
    UiProjectionBinding, UiProjectionBindingCompatibilityProof, UiProjectionBindingStopKind,
    UiProjectionBindingStopReceipt, UiProjectionConsumptionBudget,
    UiProjectionConsumptionBudgetError, UiProjectionConsumptionLimits,
    UiProjectionFieldRequirement, UiProjectionFieldRequirementError,
    UiProjectionLifecycleRequirement, UiProjectionNativeFamily, UiProjectionObservation,
    UiProjectionRetainedActivityKind, UiProjectionRetainedActivityReceipt, UiProjectionShape,
    UiProjectionUnavailableKind, UiProjectionUnavailableReceipt,
    UiQueryIdentityReportingProjection, UiQueryObservationReportingProjection,
    UiScalarProjectionBinding, UiScalarProjectionBindingAdmission, UiScalarProjectionObservation,
    UiScalarProjectionRegistration, UiScalarSchemaRequirement, WorthUiApplicationSchema,
    WorthUiInstalledQueryDomain, WorthUiInstalledQueryView, WorthUiInstalledSnapshotQueryView,
    WorthUiPresentationAsyncHostCompletion, WorthUiPresentationAsyncHostPlan,
    WorthUiPresentationAsyncInstallation, WorthUiPresentationAsyncInstallationError,
    WorthUiPresentationQueryHostInstallationRequest, WorthUiProjectionField,
    WorthUiQueryBindingRegistrationDenial, WorthUiQueryBindingRegistrationDenialKind,
    WorthUiQueryViewDeclarationDenial, WorthUiQueryViewDefinition, WorthUiQueryViewIdentity,
    WorthUiQueryViewIdentityError, WorthUiQueryViewLifecycle, WorthUiQueryViewRegistration,
    WorthUiQueryViewShape, WorthUiRecord, WorthUiScalarProjectionActionEvidence,
    WorthUiScalarProjectionActionPreconditionDenial, WorthUiScalarProjectionSourceRecord,
    WorthUiStatusActionExecution, WorthUiStatusActionOutcome, WorthUiStatusActionRequest,
    WorthUiStatusSourceOwner,
};

#[cfg(any(test, feature = "certification-support"))]
pub use worth_ui_runtime::facade::query_binding::{
    WorthUiQueryHostInstallationRequest, WorthUiScalarProjectionActionAdvance,
    WorthUiScalarProjectionActionDenied, WorthUiScalarProjectionActionExecution,
    WorthUiScalarProjectionActionIndeterminate, WorthUiScalarProjectionActionInstallation,
    WorthUiScalarProjectionActionLiveOwner, WorthUiScalarProjectionActionOutcome,
    WorthUiScalarProjectionActionPublicationCompletion, WorthUiScalarProjectionActionRequest,
    WorthUiScalarProjectionAdvance, WorthUiScalarProjectionAdvanceError,
    WorthUiScalarProjectionHostCompletion, WorthUiScalarProjectionHostPlan,
    WorthUiScalarProjectionInstallation, WorthUiScalarProjectionInstallationError,
    WorthUiScalarProjectionLiveOwner, WorthUiScalarProjectionPublicationCompletion,
    WorthUiScalarProjectionSourceCloseError, WorthUiScalarProjectionSourceCloseReceipt,
};

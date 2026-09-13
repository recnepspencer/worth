mod action_contract;
mod backend;
mod bridge;
mod installation;
mod source_lifecycle;
mod source_record;
mod support_contract;

pub use installation::{
    WorthUiQueryHostInstallationRequest, WorthUiScalarProjectionHostCompletion,
    WorthUiScalarProjectionHostPlan, WorthUiScalarProjectionInstallationError,
};
pub use source_lifecycle::{
    WorthUiScalarProjectionActionAdvance, WorthUiScalarProjectionActionDenied,
    WorthUiScalarProjectionActionEvidence, WorthUiScalarProjectionActionExecution,
    WorthUiScalarProjectionActionIndeterminate, WorthUiScalarProjectionActionInstallation,
    WorthUiScalarProjectionActionLiveOwner, WorthUiScalarProjectionActionOutcome,
    WorthUiScalarProjectionActionPreconditionDenial,
    WorthUiScalarProjectionActionPublicationCompletion, WorthUiScalarProjectionActionRequest,
    WorthUiScalarProjectionAdvance, WorthUiScalarProjectionAdvanceError,
    WorthUiScalarProjectionInstallation, WorthUiScalarProjectionLiveOwner,
    WorthUiScalarProjectionPublicationCompletion, WorthUiScalarProjectionSourceCloseError,
    WorthUiScalarProjectionSourceCloseReceipt,
};
pub use source_record::WorthUiScalarProjectionSourceRecord;

pub(crate) use backend::{
    configure_product_projection_backend, shared_source_state, SharedSourceState,
};
pub(crate) use installation::projection_runtime_builder;
pub(crate) use support_contract::evaluate_product_projection_support;

use crate::{UiAppearanceInspectionSupport, UiAppearanceInspectionWorld};

mod denial;
pub use denial::{
    UiPointerAffordanceInspectionConfirmationStop, UiPointerAffordanceInspectionPresentationDenial,
    UiPointerAffordanceInspectionTargetDenial, UiPointerAffordanceInspectionTargetingDenial,
};

/// Inert explanation of the pointer owner's sealed decision. None of these
/// values authorize input, intent execution, or publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiPointerAffordanceInspectionExplanation {
    pub world: UiAppearanceInspectionWorld,
    pub pointer_identity: u64,
    pub mounted_instance_identity: u64,
    pub node_receipt_identity: u64,
    pub presentation_epoch: u64,
    pub source_basis: u64,
    pub observation_turn: u64,
    pub graph_node_digest: Option<u64>,
    pub route: Option<Box<str>>,
    pub family: UiPointerAffordanceInspectionFamily,
    pub support: UiAppearanceInspectionSupport,
    pub decision: UiPointerAffordanceInspectionDecision,
    /// Compares sealed meaning with accepted mounted output. This is not an
    /// assertion that pending meaning has already reached the host.
    pub presentation: UiPointerAffordanceInspectionPresentation,
    pub pointer_rows_examined: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionFamily {
    Default,
    Activation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionPresentation {
    Current,
    Pending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionDecision {
    Product {
        contract_identity: Box<str>,
        causes: Box<[UiPointerAffordanceInspectionInoperableCause]>,
        selected_dependencies_visited: usize,
    },
    Confirmation {
        eligible: bool,
        stop: Option<UiPointerAffordanceInspectionConfirmationStop>,
        /// Owner-issued stop detail, materialized only by this diagnostic query.
        stop_detail: Option<Box<str>>,
        expiry_wake_millis: Option<u64>,
        slots_inspected: usize,
    },
    Unavailable(UiPointerAffordanceInspectionUnavailable),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionInoperableCause {
    Unsupported,
    WrongWorld,
    RebindRequired,
    StaleTarget,
    PolicyDenied,
    Occupied,
    Readonly,
    Pending,
    ConfirmationRequired { policy_identity: Box<str> },
}

/// The category remains machine-readable; detail preserves the owning
/// subsystem's specific denial without importing its runtime authority types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionUnavailable {
    Target {
        cause: UiPointerAffordanceInspectionTargetDenial,
        detail: Box<str>,
    },
    Presentation {
        cause: UiPointerAffordanceInspectionPresentationDenial,
        detail: Box<str>,
    },
    MissingActivationRoute,
    ConfirmationTimeUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionOutcome {
    Found(UiPointerAffordanceInspectionExplanation),
    Expired(UiPointerAffordanceInspectionExpiry),
    Unsupported,
    Unavailable,
    WrongWorld,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerAffordanceInspectionExpiry {
    GenerationChanged,
    BindingChanged,
    PresentationChanged,
    TargetChanged,
    SurfaceInvalidated,
    ObservationNotAdmitted,
}

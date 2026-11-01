//! Shared document borrows can prepare independent owned navigation attempts.
//!
//! Authored metadata and a branch description cannot forge a current selection seal.
//! ```compile_fail,E0451
//! use worth_query_publication::facade::application_entry::{WorthQueryHistorySelection, WorthQueryHistoryStep};
//! use worth_query_execution::facade::product::WorthQueryProductBranchReadIdentity;
//! fn forge(step: WorthQueryHistoryStep, physical: WorthQueryProductBranchReadIdentity) -> WorthQueryHistorySelection {
//!     WorthQueryHistorySelection { step, physical }
//! }
//! ```
//! Keep a description by cloning an issued selection.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryHistorySelection;
//! fn keep(selection: &WorthQueryHistorySelection) -> WorthQueryHistorySelection { selection.clone() }
//! ```
//!
//! Supplying a target alone cannot skip the expected-selection stage.
//! ```compile_fail,E0599
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryCheckoutTarget;
//! use worth_query_declaration::facade::{application_schema::ApplicationSchema,
//!     application_history::{ApplicationHistoryBinding, ApplicationHistoryNavigationBinding}};
//! fn skip_expected<S, H, N>(target: WorthQueryHistoryCheckoutTarget<'_, '_, S, H, N>)
//! where S: ApplicationSchema, H: ApplicationHistoryBinding<S>, N: ApplicationHistoryNavigationBinding<S, H>
//! { let _ = target.idempotency(1); }
//! ```
//! A complete description must still be admitted before it can advance.
//! ```compile_fail,E0599
//! use worth_query_publication::facade::application_entry::{WorthQueryHistoryCheckoutDescription, WorthQueryApplicationRequest};
//! use worth_query_declaration::facade::{application_schema::ApplicationSchema,
//!     application_history::{ApplicationHistoryBinding, ApplicationHistoryNavigationBinding}};
//! fn skip_admission<S, H, N>(description: WorthQueryHistoryCheckoutDescription<'_, '_, S, H, N>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//! where S: ApplicationSchema, H: ApplicationHistoryBinding<S>, N: ApplicationHistoryNavigationBinding<S, H>
//! { let _ = description.advance(request); }
//! ```
//! Obtain the actual selection from a fresh document read and complete the description.
//! ```
//! use worth_query_publication::facade::application_entry::{WorthQueryHistoryNavigationBuilder,
//!     WorthQueryHistoryCheckoutDescription, WorthQueryHistoryNodeId, WorthQueryHistorySelection};
//! use worth_query_declaration::facade::{application_schema::ApplicationSchema,
//!     application_history::{ApplicationHistoryBinding, ApplicationHistoryNavigationBinding}};
//! fn describe<'d, 'r, S, H, N>(builder: WorthQueryHistoryNavigationBuilder<'d, 'r, S, H, N>, node: WorthQueryHistoryNodeId, expected: WorthQueryHistorySelection)
//!     -> WorthQueryHistoryCheckoutDescription<'d, 'r, S, H, N>
//! where S: ApplicationSchema, H: ApplicationHistoryBinding<S>, N: ApplicationHistoryNavigationBinding<S, H>
//! { builder.checkout(node).expected_selection(expected).idempotency(1) }
//! ```
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::{ApplicationHistoryModel, ApplicationHistoryNavigationBinding},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::{
    facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
    publication_boundary::document_history::WorthQueryDocumentHistoryNavigation,
};

mod builder;
mod control;
mod progression;
mod read;
pub use builder::{
    WorthQueryHistoryCheckoutDescription, WorthQueryHistoryCheckoutTarget,
    WorthQueryHistoryExpectedCheckout, WorthQueryHistoryNavigationBuilder,
};
pub use worth_query_execution::publication_boundary::document_history::{
    WorthQueryHistoryNavigationCompletion, WorthQueryHistoryNavigationDenial,
    WorthQueryHistoryNavigationEligibilityDenial, WorthQueryHistoryNavigationPosture,
    WorthQueryHistorySelection,
};

#[derive(Debug)]
pub enum WorthQueryHistoryNavigationAdvanceDenial {
    Outputs(WorthQueryHistoryOutputDenial),
    History(WorthQueryApplicationHistoryDenial),
    Navigation(WorthQueryHistoryNavigationDenial),
}

#[must_use = "retain the owned navigation after pending work or refusal"]
pub enum WorthQueryHistoryNavigationAdvance<Navigation> {
    Selected(WorthQueryHistoryNavigationCompletion),
    Pending(Navigation),
    Refused {
        cause: WorthQueryHistoryNavigationAdvanceDenial,
        owner: Navigation,
    },
}

/// One attempt borrows the original document while its real custody stays in
/// that document's registry. Dropping a handle cannot orphan a candidate.
#[must_use = "advance or discard this navigation; document close drains registered custody"]
pub struct WorthQueryHistoryNavigation<
    'document,
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: &'document WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    navigation: WorthQueryDocumentHistoryNavigation<'runtime, S>,
    key: u128,
    marker: std::marker::PhantomData<fn() -> N>,
}

impl<S, H, N, P, Outputs> std::fmt::Debug
    for WorthQueryHistoryNavigation<'_, '_, S, H, N, P, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryNavigation")
            .field("posture", &self.navigation.posture())
            .finish_non_exhaustive()
    }
}

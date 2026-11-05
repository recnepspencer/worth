//! Ordinary model attachment and edits retain actual head and completion owners.
//!
//! A consumer cannot rewrite a sealed occurrence's parentage.
//! ```compile_fail,E0451
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryStep;
//! fn forge_parent(step: WorthQueryHistoryStep) -> WorthQueryHistoryStep {
//!     WorthQueryHistoryStep { parent: None, ..step }
//! }
//! ```
//! Read the issued descriptor or copy its descriptive metadata instead.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryStep;
//! fn keep_description(step: &WorthQueryHistoryStep) -> WorthQueryHistoryStep {
//!     let _parent = step.parent();
//!     step.clone()
//! }
//! ```
//! A closing document has only disposal custody, even while another head remains.
//! ```compile_fail,E0599
//! use worth_query_publication::facade::application_entry::{
//!     WorthQueryApplicationRequest, WorthQueryDocumentHistoryClosing,
//! };
//! use worth_query_declaration::facade::application_schema::ApplicationSchema;
//! fn read_closing<S: ApplicationSchema>(
//!     closing: WorthQueryDocumentHistoryClosing<'_, S>,
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//! ) {
//!     closing.read(request);
//! }
//! ```
//! Retry that same closing owner with the owning runtime's phase.
//! ```
//! use worth_query_publication::facade::application_entry::{
//!     WorthQueryDocumentHistoryClosing, WorthQueryDocumentHistoryCloseReceipt,
//! };
//! use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
//! use worth_query_declaration::facade::application_schema::ApplicationSchema;
//! fn retry<'runtime, S: ApplicationSchema>(
//!     closing: WorthQueryDocumentHistoryClosing<'runtime, S>,
//!     phase: &WorthQueryAdvancementPhase<'_>,
//! ) -> Result<WorthQueryDocumentHistoryCloseReceipt, WorthQueryDocumentHistoryClosing<'runtime, S>> {
//!     closing.retry(phase)
//! }
//! ```
//! A shared document borrow permits reads, but cannot transfer edit custody.
//! ```compile_fail,E0507
//! use worth_query_publication::facade::application_entry::WorthQueryApplicationHistoryDocument;
//! use worth_query_declaration::facade::{
//!     application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
//!     application_operation::{ApplicationMutationIntent, ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource},
//!     application_schema::ApplicationSchema,
//! };
//! fn edit_borrowed<S, H, I>(document: &WorthQueryApplicationHistoryDocument<'_, S, H>, intent: I)
//! where S: ApplicationSchema, H: ApplicationHistoryBinding<S>, I: ApplicationMutationIntent<S>,
//!     I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
//! { let _ = document.edit(intent); }
//! ```
//! Transfer the owned document to the builder; cancellation returns that owner.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryApplicationHistoryDocument;
//! use worth_query_declaration::facade::{
//!     application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
//!     application_operation::{ApplicationMutationIntent, ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource},
//!     application_schema::ApplicationSchema,
//! };
//! fn edit_owned<'runtime, S, H, I>(document: WorthQueryApplicationHistoryDocument<'runtime, S, H>, intent: I)
//!     -> WorthQueryApplicationHistoryDocument<'runtime, S, H>
//! where S: ApplicationSchema, H: ApplicationHistoryBinding<S>, I: ApplicationMutationIntent<S>,
//!     I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
//! { document.edit(intent).cancel() }
//! ```
mod assessment;
mod attachment;
mod close;
mod commands;
pub use commands::WorthQueryHistoryCommands;
mod context;
mod denial;
mod edit;
mod edit_builder;
mod group;
pub use group::{
    WorthQueryGroupedHistoryDocument, WorthQueryHistoryGroupCompletionIter,
    WorthQueryHistoryGroupCompletionKind, WorthQueryHistoryGroupCompletions,
    WorthQueryHistoryGroupDenial, WorthQueryHistoryGroupExecution, WorthQueryHistoryGroupFinish,
    WorthQueryHistoryGroupPosture,
};
#[doc(hidden)]
pub use group::{
    WorthQueryHistoryGroupHandoffPauseForTest, WorthQueryHistoryGroupHandoffPointForTest,
};
mod metadata;
mod navigation;
pub use navigation::{
    WorthQueryHistoryCheckoutDescription, WorthQueryHistoryCheckoutTarget,
    WorthQueryHistoryExpectedCheckout, WorthQueryHistoryNavigation,
    WorthQueryHistoryNavigationAdvance, WorthQueryHistoryNavigationAdvanceDenial,
    WorthQueryHistoryNavigationBuilder, WorthQueryHistoryNavigationCompletion,
    WorthQueryHistoryNavigationDenial, WorthQueryHistoryNavigationEligibilityDenial,
    WorthQueryHistoryNavigationIntent, WorthQueryHistoryNavigationPosture,
    WorthQueryHistorySelection,
};
mod observation;
mod outputs;
mod owner_handoff;
pub use outputs::{
    WorthQueryHistoryNoOutputs, WorthQueryHistoryOutputDenial, WorthQueryHistoryOutputsMode,
    WorthQueryHistoryWithOutputs,
};
pub use outputs::{
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryOutputControlDenial,
    WorthQueryHistoryOutputReadDenial, WorthQueryHistoryRootOutputAdmissionDenial,
};
mod page_pause;
mod paging;
mod payload;
pub use payload::{
    OrdinaryHistoryPayload, WorthQueryGovernedHistoryPayload, WorthQueryHistoryPayloadMode,
};
mod preview;
pub use preview::{WorthQueryApplicationHistoryPreview, WorthQueryHistoryPreviewResult};
mod read;
mod redo;
mod retention;
#[doc(hidden)]
pub use edit::WorthQueryHistoryEditHandoffPauseForTest;
pub use edit::{
    WorthQueryFixedHistoryEditOutcome, WorthQueryHistoryEdit, WorthQueryHistoryEditAdvance,
    WorthQueryHistoryEditDenial, WorthQueryHistoryEditFinish, WorthQueryHistoryEditOutcome,
    WorthQueryHistoryEditPosture,
};
pub use edit_builder::{
    WorthQueryHistoryEditBuilder, WorthQueryHistoryEditCaptureDenial,
    WorthQueryHistoryEditCaptureFailure, WorthQueryHistoryEditDescription,
};
pub use retention::WorthQueryHistoryWindowChange;

pub use close::{WorthQueryApplicationHistoryCloseFailure, WorthQueryHistoryClosingOutputs};
pub use denial::{
    WorthQueryApplicationHistoryAttachmentFailure, WorthQueryApplicationHistoryDenial,
};
pub use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial;
pub use worth_query_execution::publication_boundary::document_history::{
    WorthQueryDocumentHistoryCandidateDenial, WorthQueryDocumentHistoryCloseReceipt,
    WorthQueryDocumentHistoryClosing, WorthQueryDocumentHistoryClosingDenial,
    WorthQueryDocumentHistoryIdentity, WorthQueryHistoryCommandContextDenial,
    WorthQueryHistoryContextRefreshDenial, WorthQueryHistoryContextRefreshLimits,
    WorthQueryHistoryDescriptionContext, WorthQueryHistoryNodeId, WorthQueryHistoryPage,
    WorthQueryHistoryPageCursor, WorthQueryHistoryPageDenial, WorthQueryHistoryPageEntry,
    WorthQueryHistoryPageLimits, WorthQueryHistoryPageRow, WorthQueryHistoryParentBoundary,
    WorthQueryHistoryReleaseOutcome, WorthQueryHistoryRetentionStatus, WorthQueryHistoryStep,
};
pub use worth_query_execution::publication_boundary::document_history::{
    WorthQueryHistoryRedoChoice, WorthQueryHistoryRedoChoices,
};
pub use worth_query_installation::facade::WorthQueryDocumentHistoryInstallationDenialKind;

use super::WorthQueryApplicationRequest;
use worth_query_declaration::facade::{
    application_history::{ApplicationHistoryAccessBinding, ApplicationHistoryBinding},
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeResolution},
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::{
    facade::{
        application_contribution::WorthQueryAdvancementPhase,
        primary_graph::WorthQueryApplicationProjection,
    },
    publication_boundary::{
        document_history::{WorthQueryDocumentHistory, WorthQueryDocumentHistoryRead},
        program_publication_access,
    },
};

type Binding<S, H> = ApplicationHistoryAccessBinding<S, H>;
type Scope<S, H> = <Binding<S, H> as ApplicationQueryBinding<S>>::ScopeBinding;
type Query<S, H> = <Binding<S, H> as ApplicationQueryBinding<S>>::Query;
type Rows<S, H> = <<Binding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<S, H> = <Binding<S, H> as ApplicationQueryBinding<S>>::PrincipalIdentity;

/// Move-only frontend for one attached model and its actual application head.
/// Every read uses the original declared model access with fresh request authority.
/// ```compile_fail,E0382
/// use worth_query_publication::facade::application_entry::WorthQueryApplicationHistoryDocument;
/// use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
/// fn duplicate<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(document: WorthQueryApplicationHistoryDocument<'_, S, H>) {
///     let owner = document;
///     drop(document);
///     drop(owner);
/// }
/// ```
/// Transfer the one original owner instead.
/// ```
/// use worth_query_publication::facade::application_entry::WorthQueryApplicationHistoryDocument;
/// use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
/// fn transfer<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(document: WorthQueryApplicationHistoryDocument<'_, S, H>)
///     -> WorthQueryApplicationHistoryDocument<'_, S, H> { document }
/// ```
#[must_use = "retain the attached document or close its actual head custody"]
pub struct WorthQueryApplicationHistoryDocument<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    owner: WorthQueryDocumentHistory<'runtime, S, H>,
    access: H::Access,
    payload: Payload,
    outputs: Outputs,
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > std::fmt::Debug for WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationHistoryDocument")
            .finish_non_exhaustive()
    }
}

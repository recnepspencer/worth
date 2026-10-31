//! Ordinary admitted model attachment and reads retain Execution's actual head owner.
mod attachment;
mod close;
mod denial;
mod read;

pub use close::WorthQueryApplicationHistoryCloseFailure;
pub use denial::{
    WorthQueryApplicationHistoryAttachmentFailure, WorthQueryApplicationHistoryDenial,
};
pub use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial;
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
> {
    owner: WorthQueryDocumentHistory<'runtime, S, H>,
    access: H::Access,
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> std::fmt::Debug
    for WorthQueryApplicationHistoryDocument<'_, S, H>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationHistoryDocument")
            .finish_non_exhaustive()
    }
}

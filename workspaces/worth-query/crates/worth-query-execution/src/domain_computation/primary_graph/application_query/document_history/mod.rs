//! A pending exact model capture leaves only after ordinary query completion.
mod context;
mod metadata;
pub(in crate::domain_computation::primary_graph) use metadata::CompletedMetadataRead;
mod navigation;
mod operation;
mod output_intent;
mod output_read;
pub use output_intent::{WorthQueryHistoryOutputIntent, WorthQueryHistoryOutputNavigationDenial};
pub(in crate::domain_computation::primary_graph) use output_read::HistoryOutputControlPlan;
mod retained_permission;
pub(in crate::domain_computation::primary_graph) use retained_permission::HistoryRetainedPermissionStop;
mod paging;
mod pending;
mod preview;
mod redo;
pub use preview::WorthQueryHistoryPreviewReadAdmission;

use super::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationOneShotResult,
    WorthQueryApplicationProjection,
};
use crate::domain_computation::primary_graph::product_operation::document_history::confirm_document_head;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryAttachmentFailure, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryOwnedProductHead, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryAccessBinding, ApplicationHistoryBinding, ApplicationHistoryModel,
    },
    application_query::ApplicationQueryBinding,
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQueryBinding, WorthQueryInstalledDocumentHistory,
};

type Access<S, H> = ApplicationHistoryAccessBinding<S, H>;
type Query<S, H> = <Access<S, H> as ApplicationQueryBinding<S>>::Query;
type Rows<S, H> = <<Access<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Parameters<S, H> = <<Access<S, H> as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type Plan<'a, S, H> = WorthQueryAdmittedApplicationQueryPlan<
    'a,
    S,
    Query<S, H>,
    Parameters<S, H>,
    Rows<S, H>,
    <Access<S, H> as ApplicationQueryBinding<S>>::Principal,
    <Access<S, H> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    ApplicationHistoryModel<S, H>,
>;

/// One genuinely admitted READ and its installed history association.
/// It retains the original borrowed query context until ordinary execution finishes.
#[must_use = "execute the admitted document read through its owning runtime"]
pub struct WorthQueryDocumentHistoryRead<'a, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
{
    history: &'a WorthQueryInstalledDocumentHistory<S, H>,
    binding: &'a WorthQueryInstalledApplicationQueryBinding<S, Access<S, H>>,
    external: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
    plan: Plan<'a, S, H>,
    metadata: Option<super::disclosure::WorthQueryPendingApplicationQueryGovernance>,
}

impl<'a, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistoryRead<'a, S, H>
{
    pub fn new(
        history: &'a WorthQueryInstalledDocumentHistory<S, H>,
        binding: &'a WorthQueryInstalledApplicationQueryBinding<S, Access<S, H>>,
        external: &'a WorthQueryAuthenticatedExternalPrincipal<S>,
        plan: Plan<'a, S, H>,
    ) -> Self {
        Self {
            history,
            binding,
            external,
            plan,
            metadata: None,
        }
    }

    fn execute(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        (
            pending::PendingDocumentRead,
            WorthQueryApplicationOneShotResult<Query<S, H>, Rows<S, H>>,
        ),
        Denial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let pending = pending::PendingDocumentRead::capture(&self.plan);
        // The original binding-aware receiver also delegates ordinary queries.
        // Every failure preserves that receiver's actual material/cleanup denial.
        let read = runtime
            .execute_application_query_binding_one_shot(
                phase,
                self.binding,
                self.external,
                self.plan,
            )
            .map_err(Denial::Read)?;
        if runtime.installed_schema().binding_identity() != *self.history.binding_identity()
            || self.binding.query().identity() != self.history.access_binding().query().identity()
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        Ok((pending, read))
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn attach_document_history<'head, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        head: WorthQueryOwnedProductHead<'head, S>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<
        WorthQueryDocumentHistory<'head, S, H>,
        WorthQueryDocumentHistoryAttachmentFailure<'head, S>,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let binding = read.history.binding_identity().clone();
        let navigation_resources = read.history.navigation_resources();
        let (pending, _read) = match read.execute(self, phase) {
            Ok(completed) => completed,
            Err(denial) => return Err(WorthQueryDocumentHistoryAttachmentFailure { head, denial }),
        };
        if !std::ptr::eq(self, head.application()) {
            return Err(WorthQueryDocumentHistoryAttachmentFailure {
                head,
                denial: Denial::ForeignRuntime,
            });
        }
        if binding != self.installed_schema().binding_identity() {
            return Err(WorthQueryDocumentHistoryAttachmentFailure {
                head,
                denial: Denial::InstalledBindingMismatch,
            });
        }
        let expected = pending.identity();
        let execution = match phase.checked_execution_request_for(&self.product_runtime) {
            Ok(execution) => execution,
            Err(cause) => {
                return Err(WorthQueryDocumentHistoryAttachmentFailure {
                    head,
                    denial: Denial::Stopped(cause),
                })
            }
        };
        let application = head.application();
        let selection = match crate::domain_computation::primary_graph::product_operation::document_history::HistorySelectionCell::attach(
            &execution, head, expected, pending.product().retained_clone(), pending.retained_source(self), navigation_resources,
        ) {
            Ok(selection) => selection,
            Err((head, denial)) => return Err(WorthQueryDocumentHistoryAttachmentFailure { head, denial }),
        };
        let document = WorthQueryDocumentHistory {
            application,
            selection,
            model: pending.model(),
            binding,
            capture_pause: None,
            page_pause: None,
            context_pause: None,
            marker: std::marker::PhantomData,
        };
        match pending.confirm(phase, document.application(), document.branch(), document) {
            Ok(document) => Ok(document),
            Err((document, denial)) => Err(WorthQueryDocumentHistoryAttachmentFailure {
                head: document.selection.take().head,
                denial,
            }),
        }
    }

    pub fn read_document_history<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryApplicationOneShotResult<Query<S, H>, Rows<S, H>>, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let (pending, read) = read.execute(self, phase)?;
        if !std::ptr::eq(self, document.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if document.binding != self.installed_schema().binding_identity() {
            return Err(Denial::InstalledBindingMismatch);
        }
        if pending.model() != document.model {
            return Err(Denial::ForeignModel);
        }
        pending.confirm_selection(phase, document, read)
    }
}

//! A pending exact model capture leaves only after ordinary query completion.
mod current_head;
mod operation;
mod pending;

use super::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationOneShotResult,
    WorthQueryApplicationProjection,
};
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
        let document = WorthQueryDocumentHistory {
            head,
            model: pending.model(),
            expected,
            binding,
            marker: std::marker::PhantomData,
        };
        match pending.confirm(
            phase,
            document.head.application(),
            document.head.branch(),
            document,
        ) {
            Ok(document) => Ok(document),
            Err((document, denial)) => Err(WorthQueryDocumentHistoryAttachmentFailure {
                head: document.head,
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
        if !std::ptr::eq(self, document.head.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if document.binding != self.installed_schema().binding_identity() {
            return Err(Denial::InstalledBindingMismatch);
        }
        if pending.model() != document.model {
            return Err(Denial::ForeignModel);
        }
        if pending.identity() != document.expected {
            return Err(Denial::HeadDiverged);
        }
        pending
            .confirm(phase, self, document.head.branch(), read)
            .map_err(|(_, denial)| denial)
    }
}

//! A workflow instance's governed transition audit trail in one call: every
//! page of `read_workflow_history` at a snapshot's pinned commit, with the
//! recorded advancer, executor, proposer and approver of each transition
//! where the installed audit route discloses them.
use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_program::{ApplicationWorkflowReadBinding, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::{
    application_discovery::{
        WorthQueryWorkflowHistoryEntry, WorthQueryWorkflowHistoryPage, WorthQueryWorkflowIndexRead,
        WorthQueryWorkflowReadBasis, WorthQueryWorkflowReadTarget,
    },
    application_installation::WorthQueryWorkflowVocabulary,
};
use worth_query_publication::facade::application_entry::WorthQueryWorkflowObservationDenial;

use crate::session::WorthQuerySessionSnapshot;

/// Which transitions of which instance a workflow audit trail reads, and its
/// page bounds. `subject` and `audit` are the inputs of the workflow's
/// subject and audit read bindings.
pub struct WorthQueryWorkflowAuditRequest<'target, SubjectInput, AuditInput> {
    pub target: &'target WorthQueryWorkflowReadTarget,
    pub subject: SubjectInput,
    pub audit: AuditInput,
    pub page_width: usize,
    pub indexes: WorthQueryWorkflowIndexRead,
    pub maximum_pages: NonZeroUsize,
}

/// Whether an audit trail read every recorded transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowAuditCompleteness {
    Complete,
    /// `maximum_pages` pages were read and more transitions remain.
    PageLimitReached,
}

/// A workflow instance's recorded transitions at one pinned commit, in
/// recorded order. Each entry's `audit()` is `Visible { advancer, executor,
/// proposer, approver }` only where the audit read admitted this principal;
/// `Withheld` never means a role was absent.
pub struct WorthQueryWorkflowAuditTrail {
    pages: Vec<WorthQueryWorkflowHistoryPage>,
    completeness: WorthQueryWorkflowAuditCompleteness,
}

impl WorthQueryWorkflowAuditTrail {
    /// Every read transition, in recorded order.
    pub fn transitions(&self) -> impl Iterator<Item = &WorthQueryWorkflowHistoryEntry> {
        self.pages.iter().flat_map(|page| page.entries().iter())
    }

    /// The commit and branch every page observed.
    pub fn basis(&self) -> Option<&WorthQueryWorkflowReadBasis> {
        self.pages.first().map(WorthQueryWorkflowHistoryPage::basis)
    }

    pub const fn completeness(&self) -> WorthQueryWorkflowAuditCompleteness {
        self.completeness
    }
}

impl std::fmt::Debug for WorthQueryWorkflowAuditTrail {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowAuditTrail")
            .field("transitions", &self.transitions().count())
            .field("completeness", &self.completeness)
            .finish()
    }
}

impl<Schema: ApplicationSchema, Program> WorthQuerySessionSnapshot<'_, '_, Schema, Program> {
    /// Reads `request.target`'s recorded transitions at the pinned commit,
    /// following continuations up to `request.maximum_pages` pages:
    /// `snapshot.workflow_audit::<Spec, SubjectRead, AuditRead>(vocabulary, request)`.
    ///
    /// Every page admits the subject read and the independent audit read for
    /// the session principal afresh, exactly as `read_workflow_history`.
    pub fn workflow_audit<'w, W, Subject, Audit>(
        &self,
        workflow: WorthQueryWorkflowVocabulary<'w, Schema, W>,
        request: WorthQueryWorkflowAuditRequest<'_, Subject::Input, Audit::Input>,
    ) -> Result<WorthQueryWorkflowAuditTrail, WorthQueryWorkflowObservationDenial>
    where
        Schema: 'w,
        W: ApplicationWorkflowSpec<Schema = Schema>,
        Subject: ApplicationWorkflowReadBinding<Schema>,
        Audit: ApplicationWorkflowReadBinding<Schema>,
        Subject::Input: Clone,
        Audit::Input: Clone,
    {
        let mut pages: Vec<WorthQueryWorkflowHistoryPage> = Vec::new();
        for _ in 0..request.maximum_pages.get() {
            let after = pages
                .last()
                .and_then(WorthQueryWorkflowHistoryPage::continuation);
            if !pages.is_empty() && after.is_none() {
                return Ok(complete(pages));
            }
            let page = self.with_request(|at| {
                at.read_workflow_history::<W, Subject, Audit>(
                    workflow,
                    request.target,
                    request.subject.clone(),
                    request.audit.clone(),
                    request.page_width,
                    request.indexes,
                    after,
                )
            })?;
            pages.push(page);
        }
        let remaining = pages
            .last()
            .is_some_and(|page| page.continuation().is_some());
        if remaining {
            return Ok(WorthQueryWorkflowAuditTrail {
                pages,
                completeness: WorthQueryWorkflowAuditCompleteness::PageLimitReached,
            });
        }
        Ok(complete(pages))
    }
}

fn complete(pages: Vec<WorthQueryWorkflowHistoryPage>) -> WorthQueryWorkflowAuditTrail {
    WorthQueryWorkflowAuditTrail {
        pages,
        completeness: WorthQueryWorkflowAuditCompleteness::Complete,
    }
}

//! "What-if" preview forks of the paired Home, on its workflow Home.
//!
//! Each method is the same call on [`WorthQueryApplicationHome`] with the same
//! signature. A preview branch carries the same installed workflow vocabulary:
//! `session.on_branch(preview.branch())` requests accept
//! [`WorthQueryWorkflowApplicationHome::vocabulary`] for every workflow read
//! and lifecycle call, and adoption is the plain Home's expected-head swap.
use super::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution::facade::product::{
    WorthQueryProductBranch, WorthQueryProductBranchCloseReceipt,
    WorthQueryProductBranchCreationRecovery,
};

use crate::application_home::{
    WorthQueryPreviewAdoptFailure, WorthQueryPreviewDiscardDenial, WorthQueryPreviewDiscardFailure,
    WorthQueryPreviewFork, WorthQueryPreviewForkDenial,
};

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationHome<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// [`WorthQueryApplicationHome::fork_preview`].
    pub fn fork_preview(
        &self,
        label: impl Into<String>,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        self.home.fork_preview(label)
    }

    /// [`WorthQueryApplicationHome::fork_preview_with_scope`].
    pub fn fork_preview_with_scope(
        &self,
        label: impl Into<String>,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        self.home.fork_preview_with_scope(label, scope)
    }

    /// [`WorthQueryApplicationHome::continue_preview_recovery`].
    pub fn continue_preview_recovery(
        &self,
        label: impl Into<String>,
        recovery: WorthQueryProductBranchCreationRecovery,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryPreviewFork<'_, Schema>, WorthQueryPreviewForkDenial> {
        self.home.continue_preview_recovery(label, recovery, scope)
    }

    /// [`WorthQueryApplicationHome::adopt_preview`].
    pub fn adopt_preview<'home>(
        &'home self,
        preview: WorthQueryPreviewFork<'home, Schema>,
    ) -> Result<WorthQueryProductBranch, WorthQueryPreviewAdoptFailure<'home, Schema>> {
        self.home.adopt_preview(preview)
    }

    /// [`WorthQueryPreviewFork::discard`] against the paired Home, which a
    /// workflow Home does not lend out.
    pub fn discard_preview<'home>(
        &'home self,
        preview: WorthQueryPreviewFork<'home, Schema>,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryPreviewDiscardFailure<'home, Schema>>
    {
        preview.discard(&self.home)
    }

    /// [`WorthQueryApplicationHome::close_branch`].
    pub fn close_branch(
        &self,
        branch: WorthQueryProductBranch,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryPreviewDiscardDenial> {
        self.home.close_branch(branch)
    }
}

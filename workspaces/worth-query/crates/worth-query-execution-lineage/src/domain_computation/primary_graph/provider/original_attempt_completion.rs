//! Pure original completion slot port trait.

use super::super::application_attempt::WorthQueryPendingApplicationCommitReceipt as Pending;
use super::WorthQueryPrimaryGraphCommittedApplication as Completed;
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;

pub trait OriginalCompletionSlot:
    Send + Sync
{
    fn complete(&self, completed: Completed);
    fn take(&self) -> Option<Completed>;
    fn has_completion(&self, session: &WorthQueryProviderSessionTerminalBinding) -> bool;
    fn retain_cleanup(&self, original: Pending);
    fn original_cleanup(&self) -> &Pending;
}

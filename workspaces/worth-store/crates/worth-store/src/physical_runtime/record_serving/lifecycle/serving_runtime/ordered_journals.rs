//! The serving runtime's ordered journals: borrowed by name, with the
//! per-journal unresolved state that outlives each borrow until reopen.

use std::sync::MutexGuard;

use super::ServingPhysicalRuntime;
use crate::physical_runtime::ordered_journal::OrderedJournalState;
use crate::physical_runtime::{OrderedJournal, OrderedJournalDenial, OrderedJournalName};

impl ServingPhysicalRuntime {
    /// Borrows one of the home's ordered journals.
    pub fn ordered_journal(
        &self,
        name: OrderedJournalName,
    ) -> Result<OrderedJournal<'_>, OrderedJournalDenial> {
        if self.parts.work_runtime.health.requires_inspection() {
            return Err(OrderedJournalDenial::ServingRequiresInspection);
        }
        Ok(OrderedJournal::new(self, name))
    }

    /// Borrows the journal of Relational's canonical commits.
    pub fn canonical_commits(&self) -> Result<OrderedJournal<'_>, OrderedJournalDenial> {
        self.ordered_journal(OrderedJournalName::relational_canonical_commits())
    }

    /// Borrows the journal of Relational's branch transitions.
    pub fn branch_transitions(&self) -> Result<OrderedJournal<'_>, OrderedJournalDenial> {
        self.ordered_journal(OrderedJournalName::relational_branch_transitions())
    }

    pub(in crate::physical_runtime) fn ordered_journal_state(
        &self,
    ) -> MutexGuard<'_, OrderedJournalState> {
        self.ordered_journals
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub(crate) mod journal;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    WorthQueryProgramSupportPartialRetirementInventory, WorthQueryProgramSupportRetirementDenial,
    WorthQueryProgramSupportRetirementInventory, WorthQueryProgramSupportRoster,
};

use worth_foundational::facade::ProgramSupportStatus;
mod durable_retirement;
mod interpretation_membership;
mod original_readmission;
mod original_transfer;
use interpretation_membership::InterpretationMembership;
mod journal_publication;
mod journal_readmission;
mod publication_allocation;
use publication_allocation::SupportJournalState;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProgramSupportRetirementReceipt {
    inventory: WorthQueryProgramSupportRetirementInventory,
}

impl WorthQueryProgramSupportRetirementReceipt {
    pub const fn inventory(&self) -> &WorthQueryProgramSupportRetirementInventory {
        &self.inventory
    }
}

#[derive(Clone, Copy)]
struct SupportEntryState {
    status: ProgramSupportStatus,
    retained_interpretations: usize,
    mandatory_custody: usize,
    retirement_claimed: bool,
}

struct SupportLifecycleState {
    entries: BTreeMap<ApplicationProgramRevision, SupportEntryState>,
    journal: Option<SupportJournalState>,
}

pub struct WorthQueryProgramSupportLifecycle {
    state: Arc<Mutex<SupportLifecycleState>>,
    medium: std::sync::OnceLock<Arc<dyn journal::QueryObligationsMedium>>,
    publication_driver: Mutex<()>,
}

impl WorthQueryProgramSupportLifecycle {
    pub(super) fn installed<Schema>(roster: &WorthQueryProgramSupportRoster<Schema>) -> Self {
        let entries = roster
            .entries()
            .iter()
            .map(|entry| {
                (
                    *entry.revision(),
                    SupportEntryState {
                        status: ProgramSupportStatus::Active,
                        retained_interpretations: 0,
                        mandatory_custody: 0,
                        retirement_claimed: false,
                    },
                )
            })
            .collect();
        Self {
            state: Arc::new(Mutex::new(SupportLifecycleState {
                entries,
                journal: None,
            })),
            medium: std::sync::OnceLock::new(),
            publication_driver: Mutex::new(()),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, SupportLifecycleState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn pending_affects(
        state: &SupportLifecycleState,
        revision: &ApplicationProgramRevision,
    ) -> bool {
        state.journal.as_ref().is_some_and(|journal| {
            journal.pending.as_ref().is_some_and(|frame| {
                journal.ledger.record_mentions_revision(
                    frame.record(),
                    &worth_foundational::facade::CanonicalDigestId::new(*revision.as_bytes()),
                )
            })
        })
    }

    pub(super) fn is_active(&self, revision: &ApplicationProgramRevision) -> bool {
        self.state.lock().ok().is_some_and(|state| {
            !Self::pending_affects(&state, revision)
                && state.entries.get(revision).is_some_and(|entry| {
                    entry.status == ProgramSupportStatus::Active && !entry.retirement_claimed
                })
        })
    }

    pub(super) fn retain_interpretation(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportInterpretation> {
        self.acquire(revision, SupportUse::Interpretation)?;
        Some(WorthQueryProgramSupportInterpretation {
            state: Arc::clone(&self.state),
            revision: *revision,
            membership: Arc::new(Mutex::new(InterpretationMembership::Local)),
        })
    }

    pub(super) fn retain_custody(
        &self,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportCustody> {
        self.acquire(source, SupportUse::Custody)?;
        if source != target && self.acquire(target, SupportUse::Custody).is_none() {
            Self::release(&self.state, source, SupportUse::Custody);
            return None;
        }
        Some(WorthQueryProgramSupportCustody {
            state: Arc::clone(&self.state),
            source: *source,
            target: (source != target).then_some(*target),
        })
    }

    fn acquire(&self, revision: &ApplicationProgramRevision, usage: SupportUse) -> Option<()> {
        let mut state = self.state.lock().ok()?;
        if Self::pending_affects(&state, revision) {
            return None;
        }
        let entry = state.entries.get_mut(revision)?;
        if entry.status != ProgramSupportStatus::Active || entry.retirement_claimed {
            return None;
        }
        let count = match usage {
            SupportUse::Interpretation => &mut entry.retained_interpretations,
            SupportUse::Custody => &mut entry.mandatory_custody,
        };
        *count = count.checked_add(1)?;
        Some(())
    }

    fn release(
        state: &Arc<Mutex<SupportLifecycleState>>,
        revision: &ApplicationProgramRevision,
        usage: SupportUse,
    ) {
        let mut state = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(entry) = state.entries.get_mut(revision) else {
            return;
        };
        let count = match usage {
            SupportUse::Interpretation => &mut entry.retained_interpretations,
            SupportUse::Custody => &mut entry.mandatory_custody,
        };
        let Some(remaining) = count.checked_sub(1) else {
            return;
        };
        *count = remaining;
    }

    pub(in crate::domain_computation::primary_graph) fn begin_retirement(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramSupportRetirementAttempt, WorthQueryProgramSupportRetirementDenial>
    {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.medium.get().is_some() {
            return Err(
                WorthQueryProgramSupportRetirementDenial::RetirementInProgress {
                    revision: *revision,
                },
            );
        }
        Self::claim_retirement(&mut state, &self.state, revision)
    }
    fn claim_retirement(
        state: &mut SupportLifecycleState,
        owner: &Arc<Mutex<SupportLifecycleState>>,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramSupportRetirementAttempt, WorthQueryProgramSupportRetirementDenial>
    {
        use WorthQueryProgramSupportRetirementDenial as Denial;
        let entry = state
            .entries
            .get_mut(revision)
            .ok_or(Denial::UnrosteredProgram {
                revision: *revision,
            })?;
        if entry.retirement_claimed {
            return Err(Denial::RetirementInProgress {
                revision: *revision,
            });
        }
        match entry.status {
            ProgramSupportStatus::Active => entry.status = ProgramSupportStatus::Retiring,
            ProgramSupportStatus::Retiring => {} // Fresh authority resumes acknowledged Retiring.
            ProgramSupportStatus::Retired => {
                return Err(Denial::AlreadyRetired {
                    revision: *revision,
                })
            }
        }
        entry.retirement_claimed = true;
        Ok(WorthQueryProgramSupportRetirementAttempt {
            state: Arc::clone(owner),
            revision: *revision,
            finished: false,
        })
    }
}

#[derive(Clone, Copy)]
enum SupportUse {
    Interpretation,
    Custody,
}

pub struct WorthQueryProgramSupportInterpretation {
    membership: Arc<Mutex<InterpretationMembership>>,
    state: Arc<Mutex<SupportLifecycleState>>,
    revision: ApplicationProgramRevision,
}

impl WorthQueryProgramSupportInterpretation {
    pub(in crate::domain_computation::primary_graph) const fn revision(
        &self,
    ) -> &ApplicationProgramRevision {
        &self.revision
    }
    pub fn same_support_as(&self, other: &Self) -> bool {
        self.revision == other.revision && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl Drop for WorthQueryProgramSupportInterpretation {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = state
            .entries
            .get_mut(&self.revision)
            .expect("issued interpretation entry");
        self.membership
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .abandon(&mut entry.retained_interpretations);
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramSupportCustody {
    state: Arc<Mutex<SupportLifecycleState>>,
    source: ApplicationProgramRevision,
    target: Option<ApplicationProgramRevision>,
}

impl std::fmt::Debug for WorthQueryProgramSupportCustody {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProgramSupportCustody")
            .field("source", &self.source)
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

impl Drop for WorthQueryProgramSupportCustody {
    fn drop(&mut self) {
        WorthQueryProgramSupportLifecycle::release(&self.state, &self.source, SupportUse::Custody);
        if let Some(target) = &self.target {
            WorthQueryProgramSupportLifecycle::release(&self.state, target, SupportUse::Custody);
        }
    }
}

mod retirement;
pub use retirement::WorthQueryProgramSupportRetirementAttempt;

mod court_doors_for_test;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    WorthQueryProgramSupportPartialRetirementInventory, WorthQueryProgramSupportRetirementDenial,
    WorthQueryProgramSupportRetirementInventory, WorthQueryProgramSupportRoster,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramSupportStatus {
    Active,
    Retiring,
    Retired,
}

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
    status: WorthQueryProgramSupportStatus,
    retained_interpretations: usize,
    mandatory_custody: usize,
}

struct SupportLifecycleState {
    entries: BTreeMap<ApplicationProgramRevision, SupportEntryState>,
}

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramSupportLifecycle {
    state: Arc<Mutex<SupportLifecycleState>>,
}

impl WorthQueryProgramSupportLifecycle {
    pub(super) fn installed<Schema>(roster: &WorthQueryProgramSupportRoster<Schema>) -> Self {
        let entries = roster
            .entries()
            .iter()
            .map(|entry| {
                (
                    entry.revision().clone(),
                    SupportEntryState {
                        status: WorthQueryProgramSupportStatus::Active,
                        retained_interpretations: 0,
                        mandatory_custody: 0,
                    },
                )
            })
            .collect();
        Self {
            state: Arc::new(Mutex::new(SupportLifecycleState { entries })),
        }
    }

    pub(super) fn is_active(&self, revision: &ApplicationProgramRevision) -> bool {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.entries.get(revision).copied())
            .is_some_and(|entry| entry.status == WorthQueryProgramSupportStatus::Active)
    }

    pub(super) fn retain_interpretation(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportInterpretation> {
        self.acquire(revision, SupportUse::Interpretation)?;
        Some(WorthQueryProgramSupportInterpretation {
            lifecycle: self.clone(),
            revision: revision.clone(),
        })
    }

    pub(super) fn retain_custody(
        &self,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportCustody> {
        self.acquire(source, SupportUse::Custody)?;
        if source != target && self.acquire(target, SupportUse::Custody).is_none() {
            self.release(source, SupportUse::Custody);
            return None;
        }
        Some(WorthQueryProgramSupportCustody {
            lifecycle: self.clone(),
            source: source.clone(),
            target: (source != target).then(|| target.clone()),
        })
    }

    fn acquire(&self, revision: &ApplicationProgramRevision, usage: SupportUse) -> Option<()> {
        let mut state = self.state.lock().ok()?;
        let entry = state.entries.get_mut(revision)?;
        if entry.status != WorthQueryProgramSupportStatus::Active {
            return None;
        }
        let count = match usage {
            SupportUse::Interpretation => &mut entry.retained_interpretations,
            SupportUse::Custody => &mut entry.mandatory_custody,
        };
        *count = count.checked_add(1)?;
        Some(())
    }

    fn release(&self, revision: &ApplicationProgramRevision, usage: SupportUse) {
        let mut state = self
            .state
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
        let entry = state.entries.get_mut(revision).ok_or_else(|| {
            WorthQueryProgramSupportRetirementDenial::UnrosteredProgram {
                revision: revision.clone(),
            }
        })?;
        match entry.status {
            WorthQueryProgramSupportStatus::Active => {
                entry.status = WorthQueryProgramSupportStatus::Retiring;
            }
            WorthQueryProgramSupportStatus::Retiring => {
                return Err(
                    WorthQueryProgramSupportRetirementDenial::RetirementInProgress {
                        revision: revision.clone(),
                    },
                )
            }
            WorthQueryProgramSupportStatus::Retired => {
                return Err(WorthQueryProgramSupportRetirementDenial::AlreadyRetired {
                    revision: revision.clone(),
                })
            }
        }
        Ok(WorthQueryProgramSupportRetirementAttempt {
            lifecycle: self.clone(),
            revision: revision.clone(),
            finished: false,
        })
    }
}

#[derive(Clone, Copy)]
enum SupportUse {
    Interpretation,
    Custody,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramSupportInterpretation {
    lifecycle: WorthQueryProgramSupportLifecycle,
    revision: ApplicationProgramRevision,
}

impl Drop for WorthQueryProgramSupportInterpretation {
    fn drop(&mut self) {
        self.lifecycle
            .release(&self.revision, SupportUse::Interpretation);
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramSupportCustody {
    lifecycle: WorthQueryProgramSupportLifecycle,
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
        self.lifecycle.release(&self.source, SupportUse::Custody);
        if let Some(target) = &self.target {
            self.lifecycle.release(target, SupportUse::Custody);
        }
    }
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramSupportRetirementAttempt {
    lifecycle: WorthQueryProgramSupportLifecycle,
    revision: ApplicationProgramRevision,
    finished: bool,
}

impl WorthQueryProgramSupportRetirementAttempt {
    pub(in crate::domain_computation::primary_graph) fn inventory_unavailable(
        mut self,
        retained_program_bytes: usize,
    ) -> WorthQueryProgramSupportRetirementDenial {
        let mut state = self
            .lifecycle
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = state
            .entries
            .get_mut(&self.revision)
            .expect("a retirement attempt retains its installed program");
        let inventory = WorthQueryProgramSupportPartialRetirementInventory::inspected(
            self.revision.clone(),
            entry.retained_interpretations,
            entry.mandatory_custody,
            retained_program_bytes,
        );
        entry.status = WorthQueryProgramSupportStatus::Active;
        self.finished = true;
        WorthQueryProgramSupportRetirementDenial::InventoryUnavailable(inventory)
    }

    pub(in crate::domain_computation::primary_graph) fn finish(
        mut self,
        current_branches: usize,
        retained_program_bytes: usize,
    ) -> Result<WorthQueryProgramSupportRetirementReceipt, WorthQueryProgramSupportRetirementDenial>
    {
        let mut state = self
            .lifecycle
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let entry = state
            .entries
            .get_mut(&self.revision)
            .expect("a retirement attempt retains its installed program");
        let inventory = WorthQueryProgramSupportRetirementInventory::inspected(
            self.revision.clone(),
            current_branches,
            entry.retained_interpretations,
            entry.mandatory_custody,
            retained_program_bytes,
        );
        let denial = if current_branches != 0 {
            Some(WorthQueryProgramSupportRetirementDenial::CurrentBranches(
                inventory.clone(),
            ))
        } else if entry.retained_interpretations != 0 {
            Some(
                WorthQueryProgramSupportRetirementDenial::RetainedInterpretation(inventory.clone()),
            )
        } else if entry.mandatory_custody != 0 {
            Some(WorthQueryProgramSupportRetirementDenial::MandatoryCustody(
                inventory.clone(),
            ))
        } else {
            None
        };
        entry.status = if denial.is_some() {
            WorthQueryProgramSupportStatus::Active
        } else {
            WorthQueryProgramSupportStatus::Retired
        };
        self.finished = true;
        match denial {
            Some(denial) => Err(denial),
            None => Ok(WorthQueryProgramSupportRetirementReceipt { inventory }),
        }
    }
}

impl Drop for WorthQueryProgramSupportRetirementAttempt {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let mut state = self
            .lifecycle
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = state.entries.get_mut(&self.revision) {
            if entry.status == WorthQueryProgramSupportStatus::Retiring {
                entry.status = WorthQueryProgramSupportStatus::Active;
            }
        }
    }
}

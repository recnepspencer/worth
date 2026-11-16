//! One retained retirement claim; durable status changes require its receiver.
use super::*;
pub struct WorthQueryProgramSupportRetirementAttempt {
    pub(super) state: Arc<Mutex<QueryObligationState>>,
    pub(super) revision: ApplicationProgramRevision,
    pub(super) finished: bool,
}

impl WorthQueryProgramSupportRetirementAttempt {
    pub(in crate::domain_computation::primary_graph) fn inventory_unavailable(
        mut self,
        retained_program_bytes: usize,
    ) -> WorthQueryProgramSupportRetirementDenial {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let durable = state.journal.is_some();
        let durable_counts = state.journal.as_ref().map_or((0, 0), |journal| {
            journal
                .ledger
                .counts(&worth_foundational::facade::CanonicalDigestId::new(
                    *self.revision.as_bytes(),
                ))
        });
        let entry = state
            .entries
            .get_mut(&self.revision)
            .expect("a retirement attempt retains its installed program");
        let inventory = WorthQueryProgramSupportPartialRetirementInventory::inspected(
            self.revision,
            entry.retained_interpretations + durable_counts.0,
            entry.mandatory_custody + durable_counts.1,
            retained_program_bytes,
        );
        if !durable {
            entry.status = ProgramSupportStatus::Active;
            entry.retirement_claimed = false;
        }
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
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let durable = state.journal.is_some();
        let durable_counts = state.journal.as_ref().map_or((0, 0), |journal| {
            journal
                .ledger
                .counts(&worth_foundational::facade::CanonicalDigestId::new(
                    *self.revision.as_bytes(),
                ))
        });
        let entry = state
            .entries
            .get_mut(&self.revision)
            .expect("a retirement attempt retains its installed program");
        let inventory = WorthQueryProgramSupportRetirementInventory::inspected(
            self.revision,
            current_branches,
            entry.retained_interpretations + durable_counts.0,
            entry.mandatory_custody + durable_counts.1,
            retained_program_bytes,
        );
        let denial = if current_branches != 0 {
            Some(WorthQueryProgramSupportRetirementDenial::CurrentBranches(
                inventory.clone(),
            ))
        } else if entry.retained_interpretations + durable_counts.0 != 0 {
            Some(
                WorthQueryProgramSupportRetirementDenial::RetainedInterpretation(inventory.clone()),
            )
        } else if entry.mandatory_custody + durable_counts.1 != 0 {
            Some(WorthQueryProgramSupportRetirementDenial::MandatoryCustody(
                inventory.clone(),
            ))
        } else {
            None
        };
        if !durable {
            entry.status = if denial.is_some() {
                ProgramSupportStatus::Active
            } else {
                ProgramSupportStatus::Retired
            };
            entry.retirement_claimed = false;
        }
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
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let durable = state.journal.is_some();
        if let Some(entry) = state.entries.get_mut(&self.revision) {
            entry.retirement_claimed = false;
            if !durable && entry.status == ProgramSupportStatus::Retiring {
                entry.status = ProgramSupportStatus::Active;
            }
        }
    }
}

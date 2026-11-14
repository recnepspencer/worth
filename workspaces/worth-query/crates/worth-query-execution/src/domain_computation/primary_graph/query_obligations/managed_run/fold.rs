//! One managed-work identity survives until its recorded actual release is discharged.
use super::super::{QueryObligationDenial as Denial, QueryObligationLedger};
use super::*;
impl QueryObligationLedger {
    pub fn managed_runs(&self) -> impl Iterator<Item = &QueryManagedRunObligation> {
        self.runs.values()
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn validate_managed_run(
        &self,
        record: &QueryManagedRunRecord,
        position: u64,
    ) -> Result<(), Denial> {
        let identity = match record {
            QueryManagedRunRecord::Retain(row) => row.identity,
            QueryManagedRunRecord::Handoff { identity, .. }
            | QueryManagedRunRecord::Discharge { identity }
            | QueryManagedRunRecord::Original { identity, .. }
            | QueryManagedRunRecord::Recovery { identity, .. } => *identity,
        };
        if identity.home() != self.home() {
            return Err(Denial::ForeignHome);
        }
        match record {
            QueryManagedRunRecord::Retain(row) => {
                if identity.acquire_position().get() != position
                    || self.runs.contains_key(&identity)
                    || row.posture != QueryManagedRunPosture::Active
                    || row.original.is_some()
                    || row.recovery.is_some()
                    || row.original_attempt != row.attempt
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryManagedRunRecord::Handoff { posture, .. } => {
                let known = self.runs.get(&identity).ok_or(Denial::UnknownObligation)?;
                if known.posture.resources_released()
                    || !known.posture.keeps_terminal_route(*posture)
                {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryManagedRunRecord::Original { original, .. } => {
                let known = self.runs.get(&identity).ok_or(Denial::UnknownObligation)?;
                if original.home() != self.home() {
                    return Err(Denial::ForeignHome);
                }
                if known.original.is_some_and(|known| known != *original) {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryManagedRunRecord::Recovery { cause, .. } => {
                let known = self.runs.get(&identity).ok_or(Denial::UnknownObligation)?;
                if known.recovery.is_some_and(|known| known != *cause) {
                    return Err(Denial::ObligationSubjectMismatch);
                }
            }
            QueryManagedRunRecord::Discharge { .. } => {
                let known = self.runs.get(&identity).ok_or(Denial::UnknownObligation)?;
                if !known.posture.resources_released() {
                    return Err(Denial::ManagedRunPending { identity });
                }
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn apply_managed_run(
        &mut self,
        record: QueryManagedRunRecord,
    ) {
        match record {
            QueryManagedRunRecord::Retain(row) => {
                self.runs.insert(row.identity, row);
            }
            QueryManagedRunRecord::Handoff {
                identity,
                posture,
                attempt,
            } => {
                let known = self
                    .runs
                    .get_mut(&identity)
                    .expect("validated managed work");
                known.posture = posture;
                known.attempt = attempt;
            }
            QueryManagedRunRecord::Original { identity, original } => {
                self.runs
                    .get_mut(&identity)
                    .expect("validated managed work")
                    .original = Some(original);
            }
            QueryManagedRunRecord::Recovery { identity, cause } => {
                self.runs
                    .get_mut(&identity)
                    .expect("validated managed work")
                    .recovery = Some(cause);
            }
            QueryManagedRunRecord::Discharge { identity } => {
                self.runs.remove(&identity);
            }
        }
    }
}

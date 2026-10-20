//! No-fail finish of one selected scheduling attempt after its owner token.

use std::sync::Arc;

use super::*;

pub(in crate::domain_computation::primary_graph) struct PreparedSelectedSchedulingFinish<'a> {
    registry: &'a WorthQueryOutputDemandRegistry,
    interest: &'a WorthQueryOutputDemandInterest,
    member: Arc<required_work::RequiredWorkMembership>,
}

impl<'a> PreparedSelectedSchedulingFinish<'a> {
    pub(super) fn new(
        registry: &'a WorthQueryOutputDemandRegistry,
        interest: &'a WorthQueryOutputDemandInterest,
        member: Arc<required_work::RequiredWorkMembership>,
    ) -> Self {
        Self {
            registry,
            interest,
            member,
        }
    }

    /// The original denial stays with the immediate caller. The registry
    /// retains its diagnostic clone, which carries no cleanup authority.
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        mut performed_source: Option<WorthQueryPerformedOutputDemandSource>,
        result: &mut Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
    ) {
        let interrupted = match result {
            Ok(WorthQueryOutputSchedulingResult::NoEffect(denial)) | Err(denial) => {
                !required_stop::fails_row(denial)
            }
            Ok(_) => false,
        };
        let terminal_denial = match result {
            Ok(WorthQueryOutputSchedulingResult::NoEffect(denial)) | Err(denial) => {
                use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;
                // An interrupted scheduling attempt leaves the row for a later claim,
                // with the same caller posture as interrupted execution.
                denial.recovery_posture = if interrupted && execution_failure_is_retryable(denial) {
                    WorthQueryOutputDemandRecoveryPosture::Retryable
                } else {
                    WorthQueryOutputDemandRecoveryPosture::Terminal
                };
                (!interrupted).then(|| denial.clone())
            }
            _ => None,
        };
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut record = state
            .records
            .get_mut(&self.interest.key)
            .expect("selected scheduling Interest retains its owner record");
        if !matches!(record.state, DemandState::Scheduling)
            || !record
                .work_membership
                .as_ref()
                .is_some_and(|member| Arc::ptr_eq(member, &self.member))
        {
            return;
        }
        let terminal_cleanup = terminal_denial.is_some();
        let (released_bytes, obligations) = if interrupted {
            // Interruption before execution leaves the exact reopened Ready
            // with its owners, including its claims and retained custody.
            record.performed_source = performed_source.take();
            record.leave_refresh_unclaimed(DemandState::Admitted);
            (0, Vec::new())
        } else if let Some(retained) = terminal_denial {
            record.state = DemandState::Failed(retained);
            record.performed_source = None;
            let bytes = record.obligation_reserved_bytes();
            let obligations = std::mem::take(&mut record.performed_obligations);
            (bytes, obligations)
        } else {
            record.state = match result {
                Ok(WorthQueryOutputSchedulingResult::Scheduled) => DemandState::Scheduled,
                Ok(WorthQueryOutputSchedulingResult::Deferred) => {
                    record.performed_source = performed_source.take();
                    DemandState::Admitted
                }
                Ok(WorthQueryOutputSchedulingResult::NoEffect(_)) | Err(_) => unreachable!(),
            };
            (0, Vec::new())
        };
        record.wake.notify();
        record.finish();
        state.obligation_reserved_bytes = state
            .obligation_reserved_bytes
            .saturating_sub(released_bytes);
        if terminal_cleanup {
            state.defer_terminal_cleanup(self.member.key_arc(), 0);
        }
        drop(state);
        drop(obligations);
        drop(performed_source);
    }
}

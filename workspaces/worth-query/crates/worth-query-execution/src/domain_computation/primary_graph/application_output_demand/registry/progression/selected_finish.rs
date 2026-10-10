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

    /// The full denial stays with the immediate caller. The registry retains
    /// the exact semantic kind/posture with an empty diagnostic subject, so
    /// no new variable backing is allocated after Signal has been contacted.
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        mut performed_source: Option<WorthQueryPerformedOutputDemandSource>,
        result: &mut Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
    ) {
        let interrupted = scheduling_was_interrupted(result);
        let terminal_kind = match result {
            Ok(WorthQueryOutputSchedulingResult::NoEffect(denial)) | Err(denial) => {
                use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;
                // An interrupted scheduling attempt leaves the row for a later claim,
                // with the same caller posture as interrupted execution.
                denial.recovery_posture = if interrupted && execution_failure_is_retryable(denial) {
                    WorthQueryOutputDemandRecoveryPosture::Retryable
                } else {
                    WorthQueryOutputDemandRecoveryPosture::Terminal
                };
                Some(denial.kind())
            }
            _ => None,
        };
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
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
        let terminal_cleanup = terminal_kind.is_some() && !interrupted;
        let (released_bytes, obligations) = if interrupted {
            // Interruption before execution leaves the exact reopened Ready
            // with its owners, including its claims and retained custody.
            record.performed_source = performed_source.take();
            record.leave_refresh_unclaimed(DemandState::Admitted);
            (0, Vec::new())
        } else if let Some(kind) = terminal_kind {
            let mut retained = WorthQueryOutputDemandDenial::new(kind, "");
            retained.recovery_posture =
                crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal;
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

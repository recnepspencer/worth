mod replacement;
mod selected_begin;
mod selected_checkpoint_finish;
mod selected_finish;
pub(in crate::domain_computation::primary_graph) use selected_checkpoint_finish::{
    PreparedSelectedCheckpointFinish, SelectedCheckpointFinishStop,
};

use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn begin(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> WorthQueryOutputDemandAdvanceAdmission {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("live demand interest retains its owner record");
        begin_record(record, "output advancement claim identity exhausted")
    }

    /// The selected required wave pays the first registry transition before
    /// changing the record. The ordinary entry retains its historical class.
    pub(in crate::domain_computation::primary_graph) fn begin_admitted<'a>(
        &'a self,
        interest: &'a WorthQueryOutputDemandInterest,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<
        (
            WorthQueryOutputDemandAdvanceAdmission,
            Option<selected_finish::PreparedSelectedSchedulingFinish<'a>>,
        ),
        WorthQueryOutputDemandDenial,
    > {
        selected_begin::begin_admitted(self, interest, admission)
    }

    /// The required wave's second begin pays its exact ordered lookup and
    /// optional successor copy before changing Scheduled to Running. A racing
    /// owner that already moved the row leaves this attempt Pending.
    pub(in crate::domain_computation::primary_graph) fn begin_scheduled_admitted(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvanceAdmission, WorthQueryOutputDemandDenial> {
        // The ordered-lookup owner can construct a subject up to 55 bytes on
        // refusal. Its terminal backing and initialized copy are admitted
        // before it can run; failure of this envelope uses empty diagnostics.
        admission
            .admit_read_scratch(55)
            .map_err(empty_begin_preflight_denial)?;
        admission
            .charge_external_work(55 + 4)
            .map_err(|_| empty_begin_work_denial())?;
        if !std::sync::Arc::ptr_eq(&self.state, &interest.owner.state) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "",
            ));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&interest.key, admission)?;
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("live demand interest retains its owner record");
        let state @ DemandState::Scheduled = &mut record.state else {
            return Ok(WorthQueryOutputDemandAdvanceAdmission::Pending);
        };
        let copy_work = u64::try_from(std::mem::size_of::<Option<[u8; 32]>>())
            .map_err(|_| empty_begin_work_denial())?;
        admission
            .charge_external_work(copy_work)
            .map_err(|_| empty_begin_work_denial())?;
        Ok(start_scheduled(state, record.successor_of))
    }
}

fn begin_record(
    record: &mut DemandRecord,
    overflow_subject: &'static str,
) -> WorthQueryOutputDemandAdvanceAdmission {
    match &mut record.state {
        DemandState::Admitted => {
            record.state = DemandState::Scheduling;
            WorthQueryOutputDemandAdvanceAdmission::Schedule(record.performed_source.take())
        }
        state @ DemandState::Scheduled => start_scheduled(state, record.successor_of),
        DemandState::Scheduling | DemandState::Running => {
            WorthQueryOutputDemandAdvanceAdmission::Pending
        }
        DemandState::Output(output) => match &output.advancement {
            WorthQueryOutputAdvancement::Claimed(_) => {
                WorthQueryOutputDemandAdvanceAdmission::Pending
            }
            WorthQueryOutputAdvancement::Stopped { denial, .. } => {
                WorthQueryOutputDemandAdvanceAdmission::Failed(denial.clone())
            }
            WorthQueryOutputAdvancement::Idle => {
                if let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint {
                    return WorthQueryOutputDemandAdvanceAdmission::Ready(completion.clone());
                }
                let Some(next_claim) = output.next_claim.checked_add(1) else {
                    let denial = WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingRejected,
                            overflow_subject,
                        );
                    output.stop(denial.clone());
                    return WorthQueryOutputDemandAdvanceAdmission::Failed(denial);
                };
                output.next_claim = next_claim;
                let claim = WorthQueryOutputClaimIdentity(next_claim);
                let checkpoint = output
                    .checkpoint
                    .take()
                    .expect("idle output retains its checkpoint");
                output.advancement = WorthQueryOutputAdvancement::Claimed(claim);
                WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint }
            }
        },
        DemandState::Failed(denial) => {
            WorthQueryOutputDemandAdvanceAdmission::Failed(denial.clone())
        }
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_scheduling(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        performed_source: Option<WorthQueryPerformedOutputDemandSource>,
        result: &mut Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
    ) {
        match result {
            Ok(WorthQueryOutputSchedulingResult::NoEffect(denial)) | Err(denial) => {
                denial.recovery_posture =
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal;
            }
            _ => {}
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("scheduled demand retains its owner record");
        if demand_record_is_closed(record) {
            return;
        }
        record.state = match result {
            Ok(WorthQueryOutputSchedulingResult::Scheduled) => DemandState::Scheduled,
            Ok(WorthQueryOutputSchedulingResult::Deferred) => {
                record.performed_source = performed_source;
                DemandState::Admitted
            }
            Ok(WorthQueryOutputSchedulingResult::NoEffect(denial)) | Err(denial) => {
                drop(performed_source);
                record.performed_source = None;
                DemandState::Failed(denial.clone())
            }
        };
        let released = if matches!(record.state, DemandState::Failed(_)) {
            record.release_obligations()
        } else {
            0
        };
        let terminal = matches!(record.state, DemandState::Failed(_));
        record.wake.notify();
        let released_prerequisites =
            terminal.then(|| state.release_record_prerequisites(&interest.key));
        state.obligation_reserved_bytes = state.obligation_reserved_bytes.saturating_sub(released);
        state.remove_required_member_if_released(&interest.key);
        drop(state);
        drop(released_prerequisites);
    }

    pub(in crate::domain_computation::primary_graph) fn publish_checkpoint(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state.records.get_mut(&interest.key).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                "published demand record was released during execution",
            )
        })?;
        if let DemandState::Failed(denial) = &record.state {
            return Err(denial.clone());
        }
        if !matches!(record.state, DemandState::Running) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingRejected,
                "published output did not retain its running demand claim",
            ));
        }
        record.state = DemandState::Output(WorthQueryOutputProgress::new(checkpoint));
        record.performed_source = None;
        record.successor_of = None;
        record.wake.notify();
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn finish_execution_failure(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        denial: &mut WorthQueryOutputDemandDenial,
    ) {
        let retryable = execution_failure_is_retryable(denial);
        let rescheduled = execution_failure_reschedules(denial);
        denial.recovery_posture = if retryable {
            crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
        } else {
            crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("executing demand retains its owner record");
        if demand_record_is_closed(record) {
            return;
        }
        record.state = if rescheduled {
            DemandState::Scheduled
        } else {
            DemandState::Failed(denial.clone())
        };
        let released = if rescheduled {
            0
        } else {
            record.release_obligations()
        };
        record.wake.notify();
        let released_prerequisites =
            (!rescheduled).then(|| state.release_record_prerequisites(&interest.key));
        state.obligation_reserved_bytes = state.obligation_reserved_bytes.saturating_sub(released);
        state.remove_required_member_if_released(&interest.key);
        drop(state);
        drop(released_prerequisites);
    }

    pub(in crate::domain_computation::primary_graph) fn finish_checkpoint(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
        denial: Option<&WorthQueryOutputDemandDenial>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state.records.get_mut(&interest.key).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                "output checkpoint was released during advancement",
            )
        })?;
        let DemandState::Output(output) = &mut record.state else {
            return Err(match &record.state {
                DemandState::Failed(denial) => denial.clone(),
                _ => WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingRejected,
                    "output advancement no longer owns its registry record",
                ),
            });
        };
        match &output.advancement {
            WorthQueryOutputAdvancement::Claimed(active) if *active == claim => {}
            WorthQueryOutputAdvancement::Stopped { denial: stopped, interrupted_claim: Some(active) } if *active == claim => {
                let stopped = stopped.clone();
                output.checkpoint.get_or_insert(checkpoint);
                output.advancement = WorthQueryOutputAdvancement::Stopped {
                    denial: stopped.clone(),
                    interrupted_claim: None,
                };
                let released = record.release_obligations();
                state.obligation_reserved_bytes =
                    state.obligation_reserved_bytes.saturating_sub(released);
                let released_prerequisites = state.release_record_prerequisites(&interest.key);
                state.remove_required_member_if_released(&interest.key);
                drop(state);
                drop(released_prerequisites);
                return Err(stopped);
            }
            _ => return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingRejected,
                "output advancement claim no longer matches its registry record",
            )),
        }
        output.checkpoint = Some(checkpoint);
        output.advancement = match denial {
            Some(denial) if denial.recovery_posture()
                != crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable => {
                WorthQueryOutputAdvancement::Stopped {
                    denial: denial.clone(),
                    interrupted_claim: None,
                }
            }
            _ => WorthQueryOutputAdvancement::Idle,
        };
        let terminal = matches!(
            output.advancement,
            WorthQueryOutputAdvancement::Stopped { .. }
        );
        let released = if terminal {
            record.release_obligations()
        } else {
            0
        };
        record.wake.notify();
        let released_prerequisites =
            terminal.then(|| state.release_record_prerequisites(&interest.key));
        state.obligation_reserved_bytes = state.obligation_reserved_bytes.saturating_sub(released);
        state.remove_required_member_if_released(&interest.key);
        drop(state);
        drop(released_prerequisites);
        Ok(())
    }
}

fn start_scheduled(
    state: &mut DemandState,
    successor_of: Option<[u8; 32]>,
) -> WorthQueryOutputDemandAdvanceAdmission {
    *state = DemandState::Running;
    WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of }
}

fn empty_begin_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "",
    )
}

fn empty_begin_preflight_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    WorthQueryOutputDemandDenial::new(kind, "")
}

fn demand_record_is_closed(record: &DemandRecord) -> bool {
    matches!(
        &record.state,
        DemandState::Failed(denial)
            if denial.kind()
                == crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed
    )
}

pub(super) fn execution_failure_is_retryable(denial: &WorthQueryOutputDemandDenial) -> bool {
    denial.recovery_posture()
        == crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
        || matches!(
            denial.kind(),
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::PublicationStale
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Cancelled
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::TimedOut
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded
        )
}

/// Whether a failed execution leaves its row for a later claim. Only a stop
/// intrinsic to the row fails it for every caller; any other stays with the
/// request that met it, which still sees it as Terminal unless retryable.
pub(super) fn execution_failure_reschedules(denial: &WorthQueryOutputDemandDenial) -> bool {
    !super::required_stop::fails_row(denial)
}

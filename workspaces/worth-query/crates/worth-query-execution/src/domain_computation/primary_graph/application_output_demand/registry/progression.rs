use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_replaced_interest(
        &self,
        replaced: &WorthQueryOutputDemandInterest,
        replacement: &WorthQueryOutputDemandInterest,
        subject: &str,
    ) {
        if replaced.key != replacement.key {
            self.finish_superseded(replaced, subject);
        }
    }

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
        match &mut record.state {
            DemandState::Admitted => {
                record.state = DemandState::Scheduling;
                WorthQueryOutputDemandAdvanceAdmission::Schedule(record.performed_source.take())
            }
            DemandState::Scheduled => {
                record.state = DemandState::Running;
                WorthQueryOutputDemandAdvanceAdmission::Execute {
                    successor_of: record.successor_of,
                }
            }
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
                    if let Some(WorthQueryOutputCheckpoint::Ready(completion)) = &output.checkpoint
                    {
                        return WorthQueryOutputDemandAdvanceAdmission::Ready(completion.clone());
                    }
                    let Some(next_claim) = output.next_claim.checked_add(1) else {
                        let denial = WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingRejected,
                            "output advancement claim identity exhausted",
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
        record.wake.notify();
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
        let rescheduled = matches!(
            denial.kind(),
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::PublicationStale
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Cancelled
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::TimedOut
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded
        );
        denial.recovery_posture = if rescheduled {
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
        record.wake.notify();
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
        record.wake.notify();
        Ok(())
    }
}

fn demand_record_is_closed(record: &DemandRecord) -> bool {
    matches!(
        &record.state,
        DemandState::Failed(denial)
            if denial.kind()
                == crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed
    )
}

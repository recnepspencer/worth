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
        if let DemandState::DeliveryPending(pending) = &mut record.state {
            let pending = pending
                .take()
                .expect("pending delivery remains present outside delivery custody");
            record.state = DemandState::Delivering;
            return WorthQueryOutputDemandAdvanceAdmission::Deliver(pending);
        }
        if let DemandState::ReadinessPending(pending) = &mut record.state {
            let pending = pending
                .take()
                .expect("pending readiness remains present outside evaluation custody");
            record.state = DemandState::EvaluatingReadiness;
            return WorthQueryOutputDemandAdvanceAdmission::EvaluateReadiness(pending);
        }
        match &record.state {
            DemandState::Admitted => {
                record.state = DemandState::Scheduling;
                WorthQueryOutputDemandAdvanceAdmission::Schedule(record.performed_source.take())
            }
            DemandState::Scheduled => {
                record.state = DemandState::Running;
                WorthQueryOutputDemandAdvanceAdmission::Execute
            }
            DemandState::Scheduling
            | DemandState::Running
            | DemandState::Recovering(_)
            | DemandState::Delivering
            | DemandState::EvaluatingReadiness => WorthQueryOutputDemandAdvanceAdmission::Pending,
            DemandState::DeliveryPending(_) => unreachable!("pending delivery handled above"),
            DemandState::ReadinessPending(_) => unreachable!("pending readiness handled above"),
            DemandState::Completed(completion) => {
                let completion = completion.clone();
                record.state = DemandState::Recovering(completion.clone());
                WorthQueryOutputDemandAdvanceAdmission::Recover(completion)
            }
            DemandState::Settled(receipt) => {
                WorthQueryOutputDemandAdvanceAdmission::Settled(receipt.clone())
            }
            DemandState::Failed(denial) => {
                WorthQueryOutputDemandAdvanceAdmission::Failed(denial.clone())
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn finish_scheduling(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        performed_source: Option<WorthQueryPerformedOutputDemandSource>,
        result: &Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
    ) {
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

    pub(in crate::domain_computation::primary_graph) fn finish(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        result: &Result<Arc<WorthQueryOutputDemandSettlement>, WorthQueryOutputDemandDenial>,
    ) {
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
        record.state = match result {
            Ok(receipt) => DemandState::Settled(receipt.clone()),
            Err(denial) => DemandState::Failed(denial.clone()),
        };
        record.performed_source = None;
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn finish_execution_failure(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        denial: &WorthQueryOutputDemandDenial,
    ) {
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
        record.state = if matches!(
            denial.kind(),
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::PublicationStale
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Cancelled
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::TimedOut
                | crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded
        ) {
            DemandState::Scheduled
        } else {
            DemandState::Failed(denial.clone())
        };
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn finish_recovery(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        result: &Result<Arc<WorthQueryOutputDemandSettlement>, WorthQueryOutputDemandDenial>,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("recovering demand retains its owner record");
        if demand_record_is_closed(record) {
            return;
        }
        record.state = match result {
            Ok(settlement) => DemandState::Settled(settlement.clone()),
            Err(_) => match &record.state {
                DemandState::Recovering(completion) => DemandState::Completed(completion.clone()),
                _ => return,
            },
        };
        record.performed_source = None;
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn finish_delivery_pending(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        pending: WorthQueryPendingOutputDelivery,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("pending delivery retains its owner record");
        if demand_record_is_closed(record) {
            return;
        }
        record.state = DemandState::DeliveryPending(Some(pending));
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn finish_readiness_pending(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        pending: WorthQueryPendingOutputReadiness,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("pending readiness retains its owner record");
        if demand_record_is_closed(record) {
            return;
        }
        record.state = DemandState::ReadinessPending(Some(pending));
        record.wake.notify();
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

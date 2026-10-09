use super::ready_backing::PreparedReadyBacking;
use super::{
    admission::{accepts_semantic_join, interest, newest_semantic_key},
    supersede_predecessors, DemandState, WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRestoredAcceptedOutput {
    pub(in crate::domain_computation::primary_graph) checkpoint:
        super::WorthQueryAcceptedOutputCheckpointIdentity,
    pub(in crate::domain_computation::primary_graph) correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
    pub(in crate::domain_computation::primary_graph) observation:
        worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) source_scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) source_identity:
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    pub(in crate::domain_computation::primary_graph) observed_source_facts: std::sync::Arc<[
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact
    ]>,
    /// Only a reconstruction verified against the authentic recovered Native
    /// snapshot may populate this original-output witness.
    pub(in crate::domain_computation::primary_graph) native_output_witness: Option<
        std::sync::Arc<std::sync::OnceLock<crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness>>,
    >,
}

impl WorthQueryRestoredAcceptedOutput {
    pub(in crate::domain_computation::primary_graph) fn settlement_posture(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryOutputSettlementPosture {
        use crate::domain_computation::primary_graph::WorthQueryOutputSettlementPosture as Posture;
        match self.checkpoint.posture {
            super::WorthQueryAcceptedOutputCheckpointPosture::Performed => {
                Posture::RecoveredPerformed
            }
            super::WorthQueryAcceptedOutputCheckpointPosture::StableReused => {
                Posture::RecoveredStableReused
            }
        }
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_restored(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission_kind: super::DemandAdmissionKind,
        restored: super::WorthQueryRestoredAcceptedOutput,
        settlement: &std::sync::Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(WorthQueryOutputDemandInterest, bool), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = newest_semantic_key(&state, &requested_key, admission, accepts_semantic_join)?
            .unwrap_or_else(|| requested_key.clone());
        admission.charge_external_work(4).map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                "required work activation exceeds request work",
            )
        })?;
        let required_member = state.prepare_required_member(&key)?;
        state.charge_record_lookup(&key, admission)?;
        if let Some(record) = state.records.get_mut(&key) {
            if record.product_occurrence != product_occurrence
                || record.source_scope != Some(source_scope)
            {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "restored output identity belongs to another source scope or occurrence",
                ));
            }
            record.interests = record.interests.saturating_add(1);
            record.required_interests += usize::from(admission_kind.is_required());
            let interest = interest(self, key, record, admission_kind.is_required());
            state.install_required_member(required_member);
            return Ok((interest, false));
        }
        let pending_member = required_member
            .as_ref()
            .map_or(0, |member| member.reserved_bytes());
        let prepared_ready = PreparedReadyBacking::prepare(&state, admission, pending_member)?;
        let mut prepared_record = state.prepare_new_record(
            &key,
            None,
            product_occurrence,
            source_scope,
            None,
            admission,
        )?;
        state.charge_record_lookup_after_insert(&key, admission)?;
        let posting = state.prepare_restored_posting(&key, settlement, admission)?;
        if let Err(denial) = supersede_predecessors(&mut state, &requested_key, admission) {
            state.defer_cancelled_settlement_vacancy(posting.cleanup);
            return Err(denial);
        }
        let resources = restored.checkpoint.resources;
        let completion = super::WorthQueryCompletedOutputDemand {
            authority: super::WorthQueryAcceptedOutputAuthority::Restored(restored),
            readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
            resources,
        };
        prepared_record.record.interests = 1;
        prepared_record.record.required_interests = usize::from(admission_kind.is_required());
        prepared_record.record.state = DemandState::Output(
            super::WorthQueryOutputProgress::restored(prepared_ready.complete(completion)),
        );
        prepared_record.record.settlements = posting.settlements;
        state.install_prepared_record(prepared_record);
        state.required_reserved_bytes += posting.retained_bytes;
        super::settlement_index::SettlementIndex::fill_prepared(
            &posting.posting,
            settlement,
            std::sync::Arc::clone(settlement),
            std::sync::Arc::new(key.clone()),
        );
        let record = state
            .records
            .get(&key)
            .expect("the restored output record was inserted");
        let interest = interest(self, key, record, admission_kind.is_required());
        state.install_required_member(required_member);
        Ok((interest, true))
    }
}

/// A restored row's posting, prepared before the row is installed.
struct PreparedRestoredPosting {
    posting: super::settlement_index::Posting,
    cleanup: Box<super::settlement_index::PendingVacancyCleanup>,
    settlements: Vec<(std::sync::Arc<RecordedSettlementIdentity>, usize)>,
    retained_bytes: usize,
}

impl super::DemandRegistryState {
    /// No publication of this registry posts a restored output's settlement,
    /// so its row posts it at admission. A dependent then claims it as it
    /// claims a published one, and retiring the row retires its marks.
    fn prepare_restored_posting(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        settlement: &std::sync::Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedRestoredPosting, WorthQueryOutputDemandDenial> {
        let capacity = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "restored output posting exceeds registry custody capacity",
            )
        };
        // The index holds one owned key behind an Arc header.
        let key_bytes = std::mem::size_of::<WorthQueryOutputDemandKey>()
            .checked_add(2 * std::mem::size_of::<usize>())
            .and_then(|bytes| bytes.checked_add(key.producer.len()))
            .ok_or_else(capacity)?;
        let mut settlements = Vec::new();
        settlements.try_reserve_exact(1).map_err(|_| capacity())?;
        let retained_bytes = settlements
            .capacity()
            .checked_mul(std::mem::size_of::<(
                std::sync::Arc<RecordedSettlementIdentity>,
                usize,
            )>())
            .and_then(|bytes| bytes.checked_add(key_bytes))
            .ok_or_else(capacity)?;
        admission
            .charge_external_work(4)
            .and_then(|()| admission.admit_read_scratch(retained_bytes as u64))
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        "restored output posting exceeds request work",
                    )
                }
                _ => capacity(),
            })?;
        let (posting, cleanup) = self.reserve_settlement_vacancy(settlement, admission)?;
        let within = self
            .required_reserved_bytes
            .checked_add(retained_bytes)
            .is_some_and(|required| self.has_required_capacity(required));
        if !within {
            self.defer_cancelled_settlement_vacancy(cleanup);
            return Err(super::required_custody::full_custody_denial(
                retained_bytes,
                self.required_budget_bytes,
            ));
        }
        settlements.push((std::sync::Arc::clone(settlement), key_bytes));
        Ok(PreparedRestoredPosting {
            posting,
            cleanup,
            settlements,
            retained_bytes,
        })
    }
}

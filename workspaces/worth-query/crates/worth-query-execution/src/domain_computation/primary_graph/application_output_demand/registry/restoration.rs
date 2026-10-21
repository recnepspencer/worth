use super::admission::accepts_semantic_join;
use super::admission::interest;
use super::admission::newest_semantic_key;
use super::ready_backing::PreparedReadyBacking;
use super::supersede_predecessors;
use super::DemandState;
use super::WorthQueryOutputDemandInterest;
use super::WorthQueryOutputDemandKey;
use super::WorthQueryOutputDemandRegistry;

use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

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
    pub(in crate::domain_computation::primary_graph) observed_source_facts:
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
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
    ) -> Result<(WorthQueryOutputDemandInterest, bool), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = newest_semantic_key(&state, &requested_key, accepts_semantic_join)?
            .unwrap_or_else(|| requested_key.clone());

        let required_member = state.prepare_required_member(&key)?;

        // Move the issued key only in the existing-row branch. The mutation
        // guard finishes inside this closure, before registry membership edits.
        let mut existing_key = Some(key);
        let existing = state
            .records
            .get_mut(existing_key.as_ref().unwrap())
            .map(|mut record| {
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
                let interest = interest(
                    self,
                    existing_key
                        .take()
                        .expect("existing row takes its issued key"),
                    &record,
                    admission_kind.is_required(),
                );
                Ok(interest)
            });
        if let Some(existing) = existing {
            let interest = existing?;
            state.install_required_member(required_member);
            return Ok((interest, false));
        }
        let key = existing_key.expect("a new row still owns its key");
        let prepared_ready = PreparedReadyBacking::new();
        let mut prepared_record =
            state.prepare_new_record(&key, None, product_occurrence, source_scope, None)?;
        let posting = state.prepare_restored_posting(settlement)?;
        if let Err(denial) = supersede_predecessors(&mut state, &requested_key) {
            state.defer_cancelled_settlement_vacancy(posting.cleanup);
            return Err(denial);
        }
        let completion = super::WorthQueryCompletedOutputDemand {
            authority: super::WorthQueryAcceptedOutputAuthority::Restored(restored),
            readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
        };
        prepared_record.record.interests = 1;
        prepared_record.record.required_interests = usize::from(admission_kind.is_required());
        prepared_record.record.state = DemandState::Output(
            super::WorthQueryOutputProgress::restored(prepared_ready.complete(completion)),
        );
        prepared_record.record.settlements = posting.settlements;
        state.install_prepared_record(prepared_record);

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
    settlements: Vec<std::sync::Arc<RecordedSettlementIdentity>>,
}

impl super::DemandRegistryState {
    /// No publication of this registry posts a restored output's settlement,
    /// so its row posts it at admission. A dependent then claims it as it
    /// claims a published one, and retiring the row retires its marks.
    fn prepare_restored_posting(
        &mut self,
        settlement: &std::sync::Arc<RecordedSettlementIdentity>,
    ) -> Result<PreparedRestoredPosting, WorthQueryOutputDemandDenial> {
        let capacity = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "restored output posting exceeds registry custody capacity",
            )
        };
        let mut settlements = Vec::new();
        settlements.try_reserve_exact(1).map_err(|_| capacity())?;
        let (posting, cleanup) = self.reserve_settlement_vacancy(settlement)?;
        settlements.push(std::sync::Arc::clone(settlement));
        Ok(PreparedRestoredPosting {
            posting,
            cleanup,
            settlements,
        })
    }
}

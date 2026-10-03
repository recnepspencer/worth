use super::ready_backing::PreparedReadyBacking;
use super::{
    admission::{accepts_semantic_join, interest, newest_semantic_key},
    supersede_predecessors, DemandState, WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
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

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_restored(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission_kind: super::DemandAdmissionKind,
        restored: super::WorthQueryRestoredAcceptedOutput,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(WorthQueryOutputDemandInterest, bool), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = newest_semantic_key(&state, &requested_key, accepts_semantic_join)
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
        supersede_predecessors(&mut state, &requested_key)?;
        let resources = restored.checkpoint.resources;
        let completion = super::WorthQueryCompletedOutputDemand {
            authority: super::WorthQueryAcceptedOutputAuthority::Restored(restored),
            producer_commit_authority: None,
            readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
            resources,
        };
        prepared_record.record.interests = 1;
        prepared_record.record.required_interests = usize::from(admission_kind.is_required());
        prepared_record.record.state = DemandState::Output(
            super::WorthQueryOutputProgress::restored(prepared_ready.complete(completion)),
        );
        state.install_prepared_record(prepared_record);
        let record = state
            .records
            .get(&key)
            .expect("the restored output record was inserted");
        let interest = interest(self, key, record, admission_kind.is_required());
        state.install_required_member(required_member);
        Ok((interest, true))
    }
}

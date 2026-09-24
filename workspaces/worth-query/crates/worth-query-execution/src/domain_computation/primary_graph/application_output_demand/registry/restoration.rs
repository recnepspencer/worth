use std::sync::{Arc, Condvar, Mutex};

use super::{
    admission::{accepts_semantic_join, interest, newest_semantic_key},
    supersede_predecessors, DemandRecord, DemandState, DemandWake, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit_restored(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        restored: super::WorthQueryRestoredAcceptedOutput,
    ) -> Result<(WorthQueryOutputDemandInterest, bool), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = newest_semantic_key(&state, &requested_key, accepts_semantic_join)
            .unwrap_or_else(|| requested_key.clone());
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
            return Ok((interest(self, key, record), false));
        }
        supersede_predecessors(&mut state, &requested_key)?;
        let resources = restored.checkpoint.resources;
        let completion = super::WorthQueryCompletedOutputDemand {
            authority: super::WorthQueryAcceptedOutputAuthority::Restored(restored),
            readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::from_restoration(),
            resources,
        };
        let record = DemandRecord {
            interests: 1,
            required: true,
            product_occurrence,
            source_scope: Some(source_scope),
            source_commits: Vec::new(),
            state: DemandState::Output(super::WorthQueryOutputProgress::restored(completion)),
            performed_source: None,
            successor_of: None,
            wake: Arc::new(DemandWake {
                generation: Mutex::new(0),
                changed: Condvar::new(),
            }),
        };
        state.records.insert(key.clone(), record);
        let record = state
            .records
            .get(&key)
            .expect("the restored output record was inserted");
        Ok((interest(self, key, record), true))
    }
}

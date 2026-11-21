//! Runtime test doors over `WorthQueryOutputDemandRegistry` retained on
//! `WorthQueryPrimaryGraphApplicationRuntime`.

use std::sync::Arc;

use worth_query_execution_lineage::facade::primary_graph::application_output_demand::registry::{
    checkpoint::WorthQueryOutputAdvancement, join_observation::{self, StableJoinPauseForTest},
    record_map, DemandState, WorthQueryAcceptedOutputAuthority,
    WorthQueryDiscoveredRetentionControlDenial, WorthQueryDiscoveredRetentionReservationForTest,
    WorthQueryOutputCheckpoint,
};
use worth_relational::facade::identity::EntityId as Root;
use worth_runtime_world::facade::CompositeCommitIdentity as Commit;

use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn joined_stable_roots_for_test(&self) -> Vec<Root> {
        join_observation::take_joined_stable_roots_for_test()
    }

    #[doc(hidden)]
    pub fn retained_source_selections_for_test(&self) -> Vec<(Root, Commit, Commit)> {
        join_observation::take_retained_source_selections_for_test()
    }

    #[doc(hidden)]
    pub fn pause_stable_join_for_test(&self, root: Root) -> StableJoinPauseForTest {
        self.runtime.output_demands.arm_join_pause(root)
    }

    #[doc(hidden)]
    pub fn registry_row_keys_for_test(&self) -> Vec<(String, Root, u64, [u8; 32])> {
        self.runtime
            .output_demands
            .state
            .lock()
            .unwrap()
            .records
            .iter()
            .map(|(key, _)| {
                (
                    key.producer.clone(),
                    key.source.root_entity_for_test(),
                    key.source.observation_generation(),
                    key.source.runtime_idempotency_identity(),
                )
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn registry_successor_claims_for_test(&self) -> usize {
        self.runtime
            .output_demands
            .state
            .lock()
            .unwrap()
            .records
            .values()
            .filter(|row| row.successor_of.is_some())
            .count()
    }

    #[doc(hidden)]
    pub fn stable_ready_sources_for_test(&self) -> Vec<(Root, Commit, [u8; 32])> {
        self.runtime
            .output_demands
            .state
            .lock()
            .unwrap()
            .records
            .iter()
            .filter_map(|(key, row)| {
                let DemandState::Output(output) = &row.state else {
                    return None;
                };
                let Some(WorthQueryOutputCheckpoint::Ready(ready)) = &output.checkpoint else {
                    return None;
                };
                let WorthQueryAcceptedOutputAuthority::Stable(stable) = &ready.authority else {
                    return None;
                };
                Some((
                    key.source.root_entity_for_test(),
                    stable.observation().selected_commit().clone(),
                    stable.idempotency_key_identity(),
                ))
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn required_refresh_stops_for_test(&self) -> Vec<(Root, WorthQueryOutputDemandDenialKind)> {
        join_observation::take_required_refresh_stops_for_test()
    }

    #[doc(hidden)]
    pub fn has_required_row_for_test(&self, root: Root) -> bool {
        let state = self
            .runtime
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.records.iter().any(|(key, record)| {
            key.source.root_entity_for_test() == root && !record.settlements.is_empty()
        })
    }

    #[doc(hidden)]
    pub fn registry_row_count_for_test(&self) -> usize {
        self.runtime
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .records
            .len()
    }

    #[doc(hidden)]
    pub fn whole_registry_rows_visited_for_test(&self) -> Option<u64> {
        record_map::whole_rows()
    }

    #[doc(hidden)]
    pub fn leave_one_readiness_attempt_for_test(&self) {
        self.runtime
            .next_output_producer_attempt
            .store(u64::MAX - 1, std::sync::atomic::Ordering::Relaxed);
    }

    #[doc(hidden)]
    pub fn required_row_stops_for_root_for_test(
        &self,
        root: Root,
    ) -> Vec<(String, Option<WorthQueryOutputDemandDenialKind>)> {
        let state = self
            .runtime
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .records
            .iter()
            .filter(|(key, _)| key.source_epoch().root_entity_for_test() == root)
            .map(|(key, record)| (format!("{key:?}"), record.required_stop.clone()))
            .collect()
    }

    #[doc(hidden)]
    pub fn queued_required_work_for_test(&self) -> usize {
        let queue = self
            .runtime
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .required_work_queue
            .clone();
        queue.map_or(0, |queue| queue.queued_count())
    }

    #[doc(hidden)]
    pub fn published_required_rows_for_test(&self) -> Vec<(String, usize, bool, String)> {
        let state = self
            .runtime
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .records
            .iter()
            .filter_map(|(key, record)| {
                let DemandState::Output(output) = &record.state else {
                    return None;
                };
                matches!(
                    output.checkpoint,
                    Some(
                        WorthQueryOutputCheckpoint::Published { .. }
                            | WorthQueryOutputCheckpoint::RecoveredPublished { .. }
                    )
                )
                .then(|| {
                    (
                        format!("{key:?}"),
                        record.interests,
                        record.held_successor.is_some(),
                        format!(
                            "framework={}, prepared={}, cleanup={}, obligations={}, source={}",
                            record.framework_required_count,
                            record.prepared_prerequisite_claims,
                            record.pending_cleanup_queued,
                            record.performed_obligations.len(),
                            record.performed_source.is_some(),
                        ),
                    )
                })
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn queued_required_rows_for_root_for_test(&self, root: Root) -> Vec<String> {
        let queue = self
            .runtime
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .required_work_queue
            .clone();
        let Some(queue) = queue else {
            return Vec::new();
        };
        let queue = queue
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut cursor = queue.head.as_ref().map(Arc::clone);
        let mut keys = Vec::new();
        while let Some(member) = cursor {
            if member.key().source_epoch().root_entity_for_test() == root {
                keys.push(format!("{:?}", member.key()));
            }
            cursor = member
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .next
                .as_ref()
                .map(Arc::clone);
        }
        keys.sort();
        keys
    }

    #[doc(hidden)]
    pub fn entered_required_selection_is_frozen_for_test(&self) -> Option<bool> {
        self.runtime
            .output_demands
            .state
            .lock()
            .unwrap()
            .last_selection_frozen
    }

    #[doc(hidden)]
    pub fn undelivered_publication_counts_for_test(&self) -> (usize, usize) {
        let state = self.runtime.output_demands.state.lock().unwrap();
        (
            state.records.publication_count_for_test(),
            state
                .records
                .values()
                .filter(|record| {
                    matches!(
                        &record.state,
                        DemandState::Output(output)
                            if !matches!(
                                output.advancement(),
                                WorthQueryOutputAdvancement::Stopped { .. }
                            )
                            && matches!(
                                output.checkpoint,
                                Some(
                                    WorthQueryOutputCheckpoint::Published { .. }
                                        | WorthQueryOutputCheckpoint::RecoveredPublished { .. }
                                )
                            )
                    )
                })
                .count(),
        )
    }
}

impl<S: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<S>
{
    pub fn refuse_next_discovered_retention_reservation_for_test(
        &self,
    ) -> Result<
        WorthQueryDiscoveredRetentionReservationForTest,
        WorthQueryDiscoveredRetentionControlDenial,
    > {
        self.runtime.output_demands.arm_retention_reservation()
    }
}

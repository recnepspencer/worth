//! The selected prefix's private resource state is constructed once and
//! consumed into a move-only admission grant.

use std::collections::{BTreeMap, BTreeSet};

use worth_execution::{CancellationToken, LeaseRequest};

use crate::data::error::SignalError;
use crate::data::request_preparation::SignalPreparationBudget;

use super::candidate_selection::CapacityDenial;
use super::{CheckedEpochAdmissionGrant, ResourceAdmission};

impl<'lease, 'authority> ResourceAdmission<'lease, 'authority> {
    pub(super) fn decline_capacity(&self, denial: CapacityDenial) -> Result<bool, SignalError> {
        if self.width == 0 {
            Err(SignalError::PreparationMemoryExhausted {
                required: Some(denial.required),
                reserved: denial.reserved,
            })
        } else {
            Ok(false)
        }
    }

    pub(in crate::logic::planner::precompute::graph_batch) fn new(
        budget: &SignalPreparationBudget,
        lease: worth_execution::ExecutionRequest<'lease, 'authority>,
        result_heap_bound: Option<u64>,
    ) -> Result<Self, SignalError> {
        let candidate_lease = lease
            .in_scope(|scoped| {
                scoped
                    .map(|lease| {
                        lease
                            .child(LeaseRequest {
                                policy: *lease.policy(),
                                deadline: None,
                                cancellation: CancellationToken::new(),
                            })
                            .map_err(SignalError::execution_admission_denied)
                    })
                    .transpose()
            })
            .map_err(SignalError::execution_scope_denied)??;
        let physical = lease
            .in_scope(worth_execution::ExecutionMemoryReservation::available_in_scope)
            .map_err(SignalError::execution_scope_denied)?;
        let available = budget
            .remaining()
            .into_iter()
            .chain(result_heap_bound.is_none().then_some(physical).flatten())
            .min();
        Ok(Self {
            result_bounds: super::result_bounds::ResultBounds::new(result_heap_bound),
            available,
            fixed: 0,
            copies: 0,
            selected: BTreeSet::new(),
            producers: BTreeSet::new(),
            consumers: BTreeSet::new(),
            consumer_causes: BTreeMap::new(),
            waiters: BTreeSet::new(),
            sources: BTreeMap::new(),
            owner_shape: 0,
            node_root_growth: 0,
            cause_transition_count: 0,
            width: 0,
            result_grant_bytes: None,
            lease,
            candidate_lease,
            apply_bases: Vec::new(),
            candidate_bases: Vec::new(),
            precompute_bases: Vec::new(),
        })
    }

    pub(in crate::logic::planner::precompute::graph_batch) fn finish(
        self,
        budget: &mut SignalPreparationBudget,
        graph: &crate::data::graph::SignalGraph,
    ) -> Result<CheckedEpochAdmissionGrant, SignalError> {
        budget.reserve_diagnostics(
            graph
                .diagnostics_state()
                .epoch_preparation_capacity_bound(self.width)?,
        )?;
        Ok(CheckedEpochAdmissionGrant::new(
            self.width,
            self.result_grant_bytes,
            self.apply_bases,
        ))
    }
}

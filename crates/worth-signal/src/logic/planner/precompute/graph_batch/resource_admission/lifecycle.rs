//! The selected prefix's private resource state is constructed once and
//! consumed into a move-only admission grant.

use std::collections::{BTreeMap, BTreeSet};

use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};

use crate::data::error::SignalError;
use crate::data::request_preparation::SignalPreparationBudget;

use super::candidate_selection::CapacityDenial;
use super::{CheckedApplyCapacity, CheckedEpochAdmissionGrant, ResourceAdmission};

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
        lease: &'lease ExecutionResourceLease<'authority>,
    ) -> Result<Self, SignalError> {
        let candidate_lease = lease
            .child(LeaseRequest {
                policy: *lease.policy(),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .map_err(SignalError::ExecutionAdmissionDenied)?;
        Ok(Self {
            available: budget.remaining(),
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
            result_grant_bytes: 0,
            declared_result_minimum: 0,
            lease,
            candidate_lease,
            apply_bases: Vec::new(),
            candidate_bases: Vec::new(),
            precompute_bases: Vec::new(),
        })
    }

    pub(in crate::logic::planner::precompute::graph_batch) fn finish(
        self,
    ) -> Result<CheckedEpochAdmissionGrant, SignalError> {
        Ok(CheckedEpochAdmissionGrant::new(
            self.width,
            self.result_grant_bytes,
            CheckedApplyCapacity::from_bases(self.apply_bases, self.result_grant_bytes)?,
        ))
    }
}

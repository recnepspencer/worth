//! Per-member capacity for the retained apply input and worker packet.
//! The resource owner chooses the common checked result grant; this owner
//! accounts for the shape of the values that grouped apply actually retains.

use std::mem::size_of;

use crate::data::comparator::VersionComparatorPolicy;
use crate::data::dependency::{DependencyEdge, DependencySnapshotEntry};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::BoundedSignalInputs;
use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};
use crate::logic::explain::RewiringDependency;

use super::{ConcurrentWorkerInput, GroupLocalTaskCommit};

pub(in crate::logic::planner) struct ApplyMemberBasis {
    policy: VersionComparatorPolicy,
    scratch_fixed_bytes: u64,
    result_fixed_bytes: u64,
}

impl ApplyMemberBasis {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::logic::planner) fn measure(
        graph: &SignalGraph,
        node: NodeId,
        declaration: &BoundedSignalInputs,
        current: &[DependencyEdge],
        scope_heap: u64,
        old_edges_heap: u64,
        capture_bytes: u64,
        policy: VersionComparatorPolicy,
        work: &mut RetainedStoragePreparation<'_>,
    ) -> Result<Self, SignalError> {
        let old_node = graph.epoch_node_clone_charge(node, work)?.bytes();
        let old_snapshot = graph
            .get_dep_snapshot(node)?
            .retained_heap_charge(work)
            .map_err(storage_capacity)?
            .bytes();
        let policy_heap = policy
            .retained_heap_charge(work)
            .map_err(storage_capacity)?
            .bytes();
        let equivalence_heap = graph
            .node_eval_config(node)?
            .output_equivalence
            .retained_heap_charge(work)
            .map_err(storage_capacity)?
            .bytes();
        let edge_count = current
            .len()
            .checked_add(declaration.as_slice().len())
            .ok_or_else(overflow)?;
        let rewiring = slots::<RewiringDependency>(edge_count)?
            .checked_add(old_edges_heap)
            .and_then(|bytes| bytes.checked_add(scope_heap))
            .ok_or_else(overflow)?;
        let new_snapshot = slots::<DependencySnapshotEntry>(declaration.as_slice().len())?
            .checked_add(scope_heap)
            .ok_or_else(overflow)?;
        let snapshot = old_snapshot.max(new_snapshot);

        // ConcurrentWorkerInput clones its retained fields once inside the
        // kernel. Packet construction can also clone old warm authority and
        // the output-equivalence key before returning its owned result.
        let scratch_fixed_bytes = slots::<ConcurrentWorkerInput>(1)?
            .checked_add(old_node.checked_mul(2).ok_or_else(overflow)?)
            .and_then(|bytes| bytes.checked_add(rewiring))
            .and_then(|bytes| bytes.checked_add(policy_heap.checked_mul(2)?))
            .and_then(|bytes| bytes.checked_add(equivalence_heap))
            .and_then(|bytes| bytes.checked_add(capture_bytes))
            .and_then(|bytes| bytes.checked_add(snapshot))
            .ok_or_else(overflow)?;
        // The commit packet moves the prepared result into its effect. The
        // remaining result-derived material is boundary/reuse evidence and a
        // pending snapshot, each allowed one complete result-sized envelope.
        let result_fixed_bytes = slots::<GroupLocalTaskCommit>(1)?
            .checked_add(old_node.checked_mul(2).ok_or_else(overflow)?)
            .and_then(|bytes| bytes.checked_add(rewiring))
            .and_then(|bytes| bytes.checked_add(policy_heap))
            .and_then(|bytes| bytes.checked_add(snapshot.checked_mul(2)?))
            .ok_or_else(overflow)?;
        Ok(Self {
            policy,
            scratch_fixed_bytes,
            result_fixed_bytes,
        })
    }

    pub(in crate::logic::planner) fn admission_bytes(&self) -> Result<u64, SignalError> {
        self.scratch_fixed_bytes
            .checked_add(self.result_fixed_bytes)
            .ok_or_else(overflow)
    }

    pub(in crate::logic::planner) fn scratch_bytes(&self, grant: u64) -> Result<u64, SignalError> {
        self.scratch_fixed_bytes
            .checked_add(grant)
            .ok_or_else(overflow)
    }

    pub(in crate::logic::planner) fn result_bytes(&self, grant: u64) -> Result<u64, SignalError> {
        self.result_fixed_bytes
            .checked_add(grant.checked_mul(3).ok_or_else(overflow)?)
            .ok_or_else(overflow)
    }

    pub(in crate::logic::planner) fn into_capacity(
        self,
        grant: u64,
    ) -> Result<(u64, u64, VersionComparatorPolicy), SignalError> {
        Ok((
            self.scratch_bytes(grant)?,
            self.result_bytes(grant)?,
            self.policy,
        ))
    }
}

fn slots<T>(count: usize) -> Result<u64, SignalError> {
    count
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)
}

fn overflow() -> SignalError {
    SignalError::invalid_input("grouped apply capacity overflow")
}

fn storage_capacity(
    denial: crate::data::retained_storage::RetainedStoragePreparationDenial,
) -> SignalError {
    SignalError::retained_storage_denied(denial)
}

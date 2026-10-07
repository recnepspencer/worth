//! What a full run under a producer records for the next run to carry.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{ChargedBytes, PartitionItemId, ReductionTree};
use worth_foundational::facade::{ExecutionReport, PartitionIdentity};

use super::super::super::request_execution::QueryMemoryReservation;
use super::super::items::{ItemDigests, Items};
use super::super::routing::ComputationPartitionRouting;
use super::observed::observe;
use super::retained::{
    CompletedComputationRun, RetainedCall, RetainedPartition, RetainedPartitions,
    WorthQueryPartitionedComputationFullCause, WorthQueryPartitionedComputationRun,
};
use super::RetainedBasisToken;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// The calls, charges and partitions of one full run, recorded as it runs.
/// A charge that did not measure leaves the run with nothing to retain.
pub(in super::super) struct FullRecording<Key, Item> {
    basis: RetainedBasisToken,
    membership: Option<(RetainedCall, Items<Item>, ItemDigests)>,
    item_keys: BTreeMap<PartitionItemId, RetainedCall>,
    routing: Option<(ComputationPartitionRouting, QueryMemoryReservation)>,
    partitions: BTreeMap<PartitionIdentity, (Arc<Key>, u64, ComputationCallCharge)>,
    measured: bool,
}

impl<Key: Send + Sync + 'static, Item> FullRecording<Key, Item> {
    /// Records a full run that retains under `basis`, when a producer runs it.
    pub(in super::super) fn new(basis: Option<RetainedBasisToken>) -> Option<Self> {
        basis.map(|basis| Self {
            basis,
            membership: None,
            item_keys: BTreeMap::new(),
            routing: None,
            partitions: BTreeMap::new(),
            measured: true,
        })
    }

    /// The membership's call and the declared work its items' digests
    /// spent, or `None` when either did not measure, with the items and
    /// their digests.
    pub(in super::super) fn membership(
        &mut self,
        call: Option<(ComputationCallCharge, u64)>,
        items: &Items<Item>,
        digests: ItemDigests,
    ) {
        match call {
            Some((charge, declared_units)) => {
                let call = RetainedCall {
                    charge,
                    declared_units,
                };
                self.membership = Some((call, Arc::clone(items), digests));
            }
            None => self.measured = false,
        }
    }

    /// One item's key call and the declared work spent deriving and routing
    /// its key, or `None` when either did not measure.
    pub(in super::super) fn item(
        &mut self,
        item: PartitionItemId,
        call: Option<(ComputationCallCharge, u64)>,
    ) {
        match call {
            Some((charge, declared_units)) => {
                self.item_keys.insert(
                    item,
                    RetainedCall {
                        charge,
                        declared_units,
                    },
                );
            }
            None => self.measured = false,
        }
    }

    /// The routing every item was routed into, and the request memory that
    /// holds it until the record that keeps it is charged.
    pub(in super::super) fn routing(
        &mut self,
        routing: ComputationPartitionRouting,
        memory: QueryMemoryReservation,
    ) {
        self.routing = Some((routing, memory));
    }

    pub(in super::super) fn partition(
        &mut self,
        identity: PartitionIdentity,
        key: &Arc<Key>,
        key_bytes: u64,
        charge: Option<ComputationCallCharge>,
    ) {
        match charge {
            Some(charge) => {
                self.partitions
                    .insert(identity, (Arc::clone(key), key_bytes, charge));
            }
            None => self.measured = false,
        }
    }

    /// The completed run: each kernel's work and the tree, when every charge
    /// measured and the kernels and combines account for all of execution's
    /// work. `tree_memory` holds the tree until it is charged.
    pub(in super::super) fn complete<Reduced>(
        self,
        mut kernel_units: BTreeMap<PartitionIdentity, u64>,
        tree: ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
        tree_memory: QueryMemoryReservation,
        reduction_work: u64,
        cause: WorthQueryPartitionedComputationFullCause,
        report: ExecutionReport,
    ) -> CompletedComputationRun
    where
        Item: Send + Sync + ChargedBytes + 'static,
        Reduced: Send + Sync + ChargedBytes + 'static,
    {
        observe(
            WorthQueryPartitionedComputationRun::Full(cause),
            Some(report),
        );
        let accounted = kernel_units
            .values()
            .try_fold(reduction_work, |sum, units| sum.checked_add(*units))
            == Some(report.charged_work());
        let partitions = self
            .partitions
            .into_iter()
            .map(|(identity, (key, key_bytes, gather))| {
                kernel_units.remove(&identity).map(|kernel_units| {
                    let partition = RetainedPartition {
                        key,
                        key_bytes,
                        gather,
                        kernel_units,
                    };
                    (identity, Arc::new(partition))
                })
            })
            .collect::<Option<BTreeMap<_, _>>>();
        let (routing, routing_memory) = self.routing.unzip();
        let typed = match (self.membership, routing, partitions) {
            (Some((membership, items, digests)), Some(routing), Some(partitions))
                if self.measured && accounted =>
            {
                Some(RetainedPartitions {
                    items,
                    digests,
                    membership,
                    item_keys: Arc::new(self.item_keys),
                    routing: Arc::new(routing),
                    partitions,
                    tree,
                })
            }
            _ => None,
        };
        let typed_bytes = typed.as_ref().and_then(RetainedPartitions::charged_bytes);
        CompletedComputationRun {
            basis: self.basis.0,
            typed: typed.map(|typed| Arc::new(typed) as Arc<dyn std::any::Any + Send + Sync>),
            typed_bytes,
            carried: None,
            tree_memory,
            routing_memory,
        }
    }
}

/// Shows a full run that retains nothing to the test observer.
pub(in super::super) fn observe_unretained(
    cause: WorthQueryPartitionedComputationFullCause,
    report: ExecutionReport,
) {
    observe(
        WorthQueryPartitionedComputationRun::Full(cause),
        Some(report),
    );
}

//! What a full run under a producer records for the next run to carry.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{ChargedBytes, PartitionItemId, ReductionTree};
use worth_foundational::facade::{ExecutionReport, PartitionIdentity};

use super::super::super::request_execution::QueryMemoryReservation;
use super::retained::{
    observe, CompletedComputationRun, RetainedCall, RetainedPartition, RetainedPartitions,
    WorthQueryPartitionedComputationFullCause, WorthQueryPartitionedComputationRun,
};
use super::RetainedBasisToken;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// The calls, charges and partitions of one full run, recorded as it runs.
/// A charge that did not measure leaves the run with nothing to retain.
pub(in super::super) struct FullRecording<Key> {
    basis: RetainedBasisToken,
    membership: Option<ComputationCallCharge>,
    item_keys: BTreeMap<PartitionItemId, RetainedCall>,
    partitions:
        BTreeMap<PartitionIdentity, (Arc<Key>, u64, Arc<[PartitionItemId]>, ComputationCallCharge)>,
    measured: bool,
}

impl<Key: Send + Sync + 'static> FullRecording<Key> {
    /// Records a full run that retains under `basis`, when a producer runs it.
    pub(in super::super) fn new(basis: Option<RetainedBasisToken>) -> Option<Self> {
        basis.map(|basis| Self {
            basis,
            membership: None,
            item_keys: BTreeMap::new(),
            partitions: BTreeMap::new(),
            measured: true,
        })
    }

    pub(in super::super) fn membership(&mut self, charge: Option<ComputationCallCharge>) {
        self.measured &= charge.is_some();
        self.membership = charge;
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

    pub(in super::super) fn partition(
        &mut self,
        identity: PartitionIdentity,
        key: &Arc<Key>,
        key_bytes: u64,
        members: &Arc<[PartitionItemId]>,
        charge: Option<ComputationCallCharge>,
    ) {
        match charge {
            Some(charge) => {
                self.partitions.insert(
                    identity,
                    (Arc::clone(key), key_bytes, Arc::clone(members), charge),
                );
            }
            None => self.measured = false,
        }
    }

    /// The completed run: its items, each kernel's work and the tree, when
    /// every charge measured and the kernels and combines account for all of
    /// execution's work. `tree_memory` holds the tree until it is charged.
    pub(in super::super) fn complete<Item, Reduced>(
        self,
        items: Arc<BTreeMap<PartitionItemId, Item>>,
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
            .map(|(identity, (key, key_bytes, members, gather))| {
                kernel_units.remove(&identity).map(|kernel_units| {
                    let partition = RetainedPartition {
                        key,
                        key_bytes,
                        members,
                        gather,
                        kernel_units,
                    };
                    (identity, Arc::new(partition))
                })
            })
            .collect::<Option<BTreeMap<_, _>>>();
        let typed = match (self.membership, partitions) {
            (Some(membership), Some(partitions)) if self.measured && accounted => {
                Some(RetainedPartitions {
                    items,
                    membership,
                    item_keys: Arc::new(self.item_keys),
                    partitions,
                    tree,
                    reduction_work,
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

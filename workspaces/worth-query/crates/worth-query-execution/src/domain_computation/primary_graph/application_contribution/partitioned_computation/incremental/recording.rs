//! What a full run under a producer records for the next run to carry.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{ChargedBytes, PartitionItemId, ReductionTree};
use worth_foundational::facade::{ExecutionReport, PartitionIdentity};

mod transport;

use super::super::super::request_execution::QueryMemoryReservation;
use super::super::items::{ItemDigests, Items};
use super::super::routing::ComputationPartitionRouting;
use super::observed::observe;
use super::retained::{
    CompletedComputationRun, RetainedCall, RetainedPartitions,
    WorthQueryPartitionedComputationFullCause, WorthQueryPartitionedComputationRunForTest,
};
use super::{CompletedComputationRetention, PriorAbsence, RetainedBasisToken};
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
    pub(in super::super) fn new(basis: RetainedBasisToken) -> Self {
        Self {
            basis,
            membership: None,
            item_keys: BTreeMap::new(),
            routing: None,
            partitions: BTreeMap::new(),
            measured: true,
        }
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

    /// The completed run keeps native kernel cost receipts and its reduced tree.
    /// A context-dependent or nested kernel leaves no reusable receipt.
    /// `tree_memory` holds the tree until it is charged.
    pub(in super::super) fn complete<Reduced>(
        self,
        results: Option<worth_execution::ExecutionRetainedRun<Reduced>>,
        tree: ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
        tree_memory: QueryMemoryReservation,
        execution: &super::super::super::request_execution::QueryRequestExecution<'_>,
        cause: WorthQueryPartitionedComputationFullCause,
        report: ExecutionReport,
    ) -> Result<
        CompletedComputationRetention,
        super::super::super::WorthQueryManagedComputationResourceDenial,
    >
    where
        Item: Send + Sync + ChargedBytes + 'static,
        Reduced: Send + Sync + ChargedBytes + 'static,
    {
        observe(
            WorthQueryPartitionedComputationRunForTest::Full(cause),
            Some(report),
        );
        let Some(native_run) = results.filter(|run| {
            run.partition_count() == self.partitions.len() && run.requires_execution().is_none()
        }) else {
            return Ok(CompletedComputationRetention::Absent(
                PriorAbsence::Unmeasured,
            ));
        };
        if !self.partitions.keys().all(|identity| {
            native_run.result(*identity).is_some_and(|result| {
                result.identity() == *identity && result.native_reuse_available()
            })
        }) {
            return Ok(CompletedComputationRetention::Absent(
                PriorAbsence::Unmeasured,
            ));
        }
        if !self.measured || self.membership.is_none() || self.routing.is_none() {
            return Ok(CompletedComputationRetention::Absent(
                PriorAbsence::Unmeasured,
            ));
        }
        let partitions = Some(super::partition_metadata::PartitionMetadata::from_recorded(
            self.partitions,
            execution,
        )?);
        let (routing, routing_memory) = self.routing.unzip();
        let typed = match (self.membership, routing, partitions) {
            (Some((membership, items, digests)), Some(routing), Some(partitions))
                if self.measured =>
            {
                RetainedPartitions {
                    items,
                    digests,
                    membership,
                    item_keys: Arc::new(self.item_keys),
                    routing: Arc::new(routing),
                    partitions,
                    native_run,
                    tree,
                }
            }
            _ => {
                return Ok(CompletedComputationRetention::Absent(
                    PriorAbsence::Unmeasured,
                ))
            }
        };
        let typed_bytes = typed.charged_bytes();
        Ok(CompletedComputationRetention::Produced(
            CompletedComputationRun {
                basis: self.basis.0,
                typed: Arc::new(typed),
                typed_bytes,
                carried: None,
                tree_memory,
                routing_memory,
            },
        ))
    }
}

/// Shows a full run that retains nothing to the test observer.
pub(in super::super) fn observe_unretained(
    cause: WorthQueryPartitionedComputationFullCause,
    report: ExecutionReport,
) {
    observe(
        WorthQueryPartitionedComputationRunForTest::Full(cause),
        Some(report),
    );
}

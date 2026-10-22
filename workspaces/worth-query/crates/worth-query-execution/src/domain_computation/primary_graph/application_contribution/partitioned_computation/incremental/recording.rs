//! What a full run under a producer records for the next run to carry.

use crate::domain_computation::primary_graph::{
    application_attempt::ComputationRead, invariant_projection::ComputationCallSequence,
};
use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{NativeReductionTree, PartitionItemId};
use worth_foundational::facade::{ExecutionReport, PartitionIdentity};

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
    routing: Option<ComputationPartitionRouting>,
    partitions: BTreeMap<PartitionIdentity, (Arc<Key>, ComputationCallCharge)>,
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
        call: Option<(ComputationCallCharge, Option<u64>)>,
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
        call: Option<(ComputationCallCharge, Option<u64>)>,
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
    pub(in super::super) fn routing(&mut self, routing: ComputationPartitionRouting) {
        self.routing = Some(routing);
    }

    pub(in super::super) fn partition(
        &mut self,
        identity: PartitionIdentity,
        key: &Arc<Key>,
        charge: Option<ComputationCallCharge>,
    ) {
        match charge {
            Some(charge) => {
                self.partitions.insert(identity, (Arc::clone(key), charge));
            }
            None => self.measured = false,
        }
    }

    /// The completed run keeps native kernel cost receipts and its reduced tree.
    /// A context-dependent or nested kernel leaves no reusable receipt.
    pub(in super::super) fn complete<Reduced, Stopped>(
        self,
        results: Option<worth_execution::ExecutionRetainedRun<Reduced>>,
        tree: NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
        execution: &super::super::super::request_execution::QueryRequestExecution<'_>,
        cause: WorthQueryPartitionedComputationFullCause,
        report: ExecutionReport,
    ) -> Result<
        CompletedComputationRetention,
        super::super::WorthQueryPartitionedComputationDenial<Stopped>,
    >
    where
        Item: Send + Sync + 'static,
        Reduced: Send + Sync + 'static,
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
        let membership = self.membership.as_ref().expect("measured membership");
        let call_sequence = ComputationCallSequence::build(
            || {
                std::iter::once((
                    ComputationRead::Membership,
                    &membership.0.charge,
                    membership.0.declared_units,
                ))
                .chain(self.item_keys.iter().map(|(item, call)| {
                    (
                        ComputationRead::ItemKey(*item),
                        &call.charge,
                        call.declared_units,
                    )
                }))
                .chain(self.partitions.iter().map(|(id, (_, charge))| {
                    (
                        ComputationRead::Partition(*id),
                        charge,
                        execution.has_accounting().then_some(0),
                    )
                }))
            },
            execution,
        )
        .map_err(super::super::WorthQueryPartitionedComputationDenial::from_call_sequence)?;
        let policy = execution
            .allocation_policy()
            .map_err(super::super::WorthQueryPartitionedComputationDenial::Resource)?;
        let item_keys = super::item_bounds::retain_calls(&self.item_keys, policy)
            .map_err(super::super::WorthQueryPartitionedComputationDenial::from_ordered)?;
        let partitions = Some(
            super::partition_metadata::PartitionMetadata::from_recorded(self.partitions, policy)
                .map_err(super::super::WorthQueryPartitionedComputationDenial::from_ordered)?,
        );
        let routing = self.routing;
        let typed = match (self.membership, routing, partitions) {
            (Some((membership, items, digests)), Some(routing), Some(partitions))
                if self.measured =>
            {
                RetainedPartitions {
                    items,
                    digests,
                    membership,
                    item_keys,
                    call_sequence: Arc::clone(&call_sequence),
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
        Ok(CompletedComputationRetention::Produced(
            CompletedComputationRun {
                basis: self.basis.0,
                typed: Arc::new(typed),
                call_sequence,
                carried: None,
                attribution_memory: execution
                    .has_accounting()
                    .then(|| {
                        execution
                            .reserve(0)
                            .map_err(super::super::WorthQueryPartitionedComputationDenial::Resource)
                    })
                    .transpose()?,
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

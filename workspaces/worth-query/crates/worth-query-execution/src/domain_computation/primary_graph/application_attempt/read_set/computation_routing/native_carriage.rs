//! What the read set asks of the native prior a partitioned computation
//! carried calls from.
//!
//! The partitioned computation owns that prior (its retained custody, the
//! snapshot its comparator checked, and the changes it rebases over). The
//! read set holds it only as this question: whether two holders are the same
//! custody at the same snapshot, the facts the custody sealed, and whether a
//! completed run closed exactly over it. Identity is compared by address,
//! never recovered by downcast.

use std::collections::BTreeSet;
use std::sync::Arc;

use worth_relational::facade::snapshots::SnapshotHandle;

use super::{NativeCallCarriage, SealedComputationFacts};
use crate::domain_computation::primary_graph::application_contribution::CompletedComputationRetention;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;

/// The native prior a projection's calls were carried from, as the read set
/// sees it. Holders share one custody through `Arc<dyn NativeCarriageReads>`.
pub(in crate::domain_computation::primary_graph) trait NativeCarriageReads: Send + Sync {
    /// The retained custody's address and the snapshot it was checked at.
    fn identity(&self) -> (*const (), &SnapshotHandle);

    /// Whether `other` holds the same retained custody at the same snapshot.
    fn same_native(&self, other: &dyn NativeCarriageReads) -> bool {
        let (prior, snapshot) = self.identity();
        let (other_prior, other_snapshot) = other.identity();
        std::ptr::eq(prior, other_prior) && snapshot == other_snapshot
    }

    /// Whether the custody was checked at exactly `snapshot`.
    fn at_snapshot(&self, snapshot: &SnapshotHandle) -> bool;

    /// The facts the retained run sealed.
    fn facts(&self) -> &SealedComputationFacts;

    /// The changes the comparator observed since the retained run, which a
    /// postcondition carry rebases over.
    fn rebase_changes(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges>;

    /// Whether `completed` closed over this exact custody at its native pin
    /// with these closed ranges.
    fn closes_native_ranges(
        &self,
        completed: &CompletedComputationRetention,
        ranges: &Arc<NativeCallCarriage>,
    ) -> bool;

    /// Whether `completed` carried exactly `calls` from this exact custody at
    /// its native pin.
    fn closes_native_reads(
        &self,
        completed: &CompletedComputationRetention,
        calls: &BTreeSet<ComputationRead>,
    ) -> bool;
}

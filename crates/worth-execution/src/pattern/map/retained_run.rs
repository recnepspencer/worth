//! Successful native runs with persistent, independently funded result slots.
mod advance;
mod issue;
#[cfg(test)]
mod tests;

use super::{MapStop, RetainedMapDenial};
use crate::{reduction::RunTree, ChargedBytes, ExecutionRetainedResult, LeaseDenial};
use worth_foundational::{ExecutionReport, PartitionIdentity};

/// Actual physical maintenance, separate from canonical kernel work/span.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RetainedRunMetrics {
    pub supplied_inputs: u64,
    pub edit_visits: u64,
    pub copied_nodes: u64,
    pub range_visits: u64,
    pub carried_ranges: u64,
    pub carried_slots: u64,
    pub carried_slot_visits: u64,
}

#[derive(Debug)]
pub enum RetainedRunDenial {
    Native(RetainedMapDenial),
    Map(super::MapMemoryOverflow),
    RequiresKeyless,
    UnknownRemoval(PartitionIdentity),
    ChangedAndRemoved(PartitionIdentity),
}
impl From<LeaseDenial> for RetainedRunDenial {
    fn from(cause: LeaseDenial) -> Self {
        Self::Native(RetainedMapDenial::Admission(cause))
    }
}

/// Issued only by actual complete native evaluation and canonical settlement.
/// It owns native results and cached costs, never input equality, source,
/// snapshot, codec, installation, principal, edition or session authority.
/// No request, lease borrow, old RunLimits or parent-run chain is retained.
///
/// ```compile_fail,E0599
/// use worth_execution::ExecutionRetainedRun;
/// let forged = ExecutionRetainedRun::new(10_u64);
/// ```
pub struct ExecutionRetainedRun<R> {
    tree: RunTree<R>,
}
impl<R> Clone for ExecutionRetainedRun<R> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
        }
    }
}
impl<R> ExecutionRetainedRun<R> {
    pub fn partition_count(&self) -> usize {
        self.tree.summary().count
    }
    pub fn charged_work(&self) -> u64 {
        self.tree.summary().work
    }
    pub fn charged_span(&self) -> u64 {
        self.tree.summary().span
    }
    pub fn result(&self, id: PartitionIdentity) -> Option<&ExecutionRetainedResult<R>> {
        self.tree.get(id)
    }
    pub fn requires_execution(&self) -> Option<PartitionIdentity> {
        self.tree.summary().unqualified
    }
}
impl<R> ChargedBytes for ExecutionRetainedRun<R> {
    fn additional_charged_bytes(&self) -> u64 {
        self.tree.bytes()
    }
}

/// Accepted canonical values of a stopped attempt. A prefix cannot advance.
pub struct ExecutionRetainedPrefix<R> {
    tree: RunTree<R>,
}
impl<R> ExecutionRetainedPrefix<R> {
    pub fn partition_count(&self) -> usize {
        self.tree.summary().count
    }
    pub fn result(&self, id: PartitionIdentity) -> Option<&ExecutionRetainedResult<R>> {
        self.tree.get(id)
    }
}
impl<R> ChargedBytes for ExecutionRetainedPrefix<R> {
    fn additional_charged_bytes(&self) -> u64 {
        self.tree.bytes()
    }
}

pub enum RetainedRunOutcome<R, E> {
    Complete {
        run: ExecutionRetainedRun<R>,
        report: ExecutionReport,
        metrics: RetainedRunMetrics,
    },
    Stopped {
        completed_prefix: ExecutionRetainedPrefix<R>,
        boundary: Option<PartitionIdentity>,
        reason: MapStop<E>,
        report: ExecutionReport,
        metrics: RetainedRunMetrics,
    },
}
impl<R, E> RetainedRunOutcome<R, E> {
    pub fn report(&self) -> ExecutionReport {
        match self {
            Self::Complete { report, .. } | Self::Stopped { report, .. } => *report,
        }
    }
    pub fn metrics(&self) -> RetainedRunMetrics {
        match self {
            Self::Complete { metrics, .. } | Self::Stopped { metrics, .. } => *metrics,
        }
    }
}

pub(in crate::pattern) fn finish_native_outcome<R, E>(
    mut tree: RunTree<R>,
    outcome: crate::backend::BatchOutcome<ExecutionRetainedResult<R>, E>,
    lease: Option<&crate::ExecutionResourceLease<'_>>,
    mut metrics: RetainedRunMetrics,
) -> Result<RetainedRunOutcome<R, E>, LeaseDenial> {
    if outcome.stop.is_some() {
        if let Some(boundary) = outcome.prefix_boundary {
            tree.prefix(boundary, lease, &mut metrics)?;
        } else {
            tree = RunTree::empty();
        }
    }
    for value in outcome.values {
        tree.put(value, lease, &mut metrics)?;
    }
    Ok(match outcome.stop {
        None => RetainedRunOutcome::Complete {
            run: ExecutionRetainedRun { tree },
            report: outcome.report,
            metrics,
        },
        Some(reason) => RetainedRunOutcome::Stopped {
            completed_prefix: ExecutionRetainedPrefix { tree },
            boundary: outcome.prefix_boundary,
            reason,
            report: outcome.report,
            metrics,
        },
    })
}

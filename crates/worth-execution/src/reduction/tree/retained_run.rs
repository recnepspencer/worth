//! Native settled slots using the ordinary Cartesian tree's edit machinery.
mod engine;
mod summary;
#[cfg(test)]
mod tests;
pub(crate) use summary::RunSummary;

use super::{edit, Link, ReductionRunStop};
use crate::{ExecutionResourceLease, ExecutionRetainedResult, LeaseDenial, RetainedRunMetrics};
use engine::{RunEngine, Slot};
use worth_foundational::PartitionIdentity;

pub(crate) struct RunTree<R> {
    root: Link<Slot<R>>,
}
impl<R> Clone for RunTree<R> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}

pub(crate) enum RunRange<'a, R> {
    Summary(RunSummary),
    Slot(&'a ExecutionRetainedResult<R>),
}

impl<R> RunTree<R> {
    pub(crate) fn empty() -> Self {
        Self { root: None }
    }
    pub(crate) fn summary(&self) -> RunSummary {
        self.root
            .as_ref()
            .map_or_else(RunSummary::empty, |n| n.aggregate.summary)
    }
    pub(crate) fn bytes(&self) -> u64 {
        self.root
            .as_ref()
            .map_or(0, |n| n.retained_bytes.expect("accounted run backing"))
    }
    pub(crate) fn get(&self, id: PartitionIdentity) -> Option<&ExecutionRetainedResult<R>> {
        edit::leaf(&self.root, id)?.result.as_ref()
    }
    pub(crate) fn contains(&self, id: PartitionIdentity, metrics: &mut RetainedRunMetrics) -> bool {
        let mut node = self.root.as_deref();
        while let Some(current) = node {
            metrics.edit_visits += 1;
            if current.identity == id {
                return true;
            }
            node = if id < current.identity {
                current.left.as_deref()
            } else {
                current.right.as_deref()
            };
        }
        false
    }
    pub(crate) fn prefix(
        &mut self,
        before: PartitionIdentity,
        lease: Option<&ExecutionResourceLease<'_>>,
        metrics: &mut RetainedRunMetrics,
    ) -> Result<(), LeaseDenial> {
        self.root =
            edit::prefix(&self.root, before, &mut RunEngine { lease, metrics }).map_err(denial)?;
        Ok(())
    }
    pub(crate) fn put(
        &mut self,
        result: ExecutionRetainedResult<R>,
        lease: Option<&ExecutionResourceLease<'_>>,
        metrics: &mut RetainedRunMetrics,
    ) -> Result<(), LeaseDenial> {
        let id = result.identity();
        let value = Slot::new(result);
        let mut engine = RunEngine { lease, metrics };
        // The membership search is part of the measured native edit, not a
        // reconstructed roster or a second mutable membership authority.
        let mut current = self.root.as_deref();
        let mut found = false;
        while let Some(node) = current {
            engine.metrics.edit_visits += 1;
            if id == node.identity {
                found = true;
                break;
            }
            current = if id < node.identity {
                node.left.as_deref()
            } else {
                node.right.as_deref()
            };
        }
        self.root = if found {
            edit::update(&self.root, id, value, &mut engine).map(|v| v.link)
        } else {
            edit::insert(&self.root, id, value, &mut engine)
        }
        .map_err(denial)?;
        Ok(())
    }
    pub(crate) fn remove(
        &mut self,
        id: PartitionIdentity,
        lease: Option<&ExecutionResourceLease<'_>>,
        metrics: &mut RetainedRunMetrics,
    ) -> Result<(), LeaseDenial> {
        let mut engine = RunEngine { lease, metrics };
        self.root = edit::delete(&self.root, id, &mut engine).map_err(denial)?;
        Ok(())
    }
    /// Walk only the boundary paths and ranges that the consumer declines.
    /// `true` accepts a cached summary; `false` descends to actual native slots.
    pub(crate) fn between(
        &self,
        after: Option<PartitionIdentity>,
        before: Option<PartitionIdentity>,
        visit: &mut impl FnMut(RunRange<'_, R>) -> bool,
    ) -> u64 {
        fn walk<R>(
            link: &Link<Slot<R>>,
            after: Option<PartitionIdentity>,
            before: Option<PartitionIdentity>,
            visit: &mut impl FnMut(RunRange<'_, R>) -> bool,
            visits: &mut u64,
        ) {
            let Some(node) = link else {
                return;
            };
            *visits += 1;
            let summary = node.aggregate.summary;
            if after.is_some_and(|id| summary.last.is_some_and(|last| last <= id))
                || before.is_some_and(|id| summary.first.is_some_and(|first| first >= id))
            {
                return;
            }
            let inside = after.is_none_or(|id| summary.first.is_some_and(|first| first > id))
                && before.is_none_or(|id| summary.last.is_some_and(|last| last < id));
            if inside && visit(RunRange::Summary(summary)) {
                return;
            }
            walk(&node.left, after, before, visit, visits);
            if after.is_none_or(|id| node.identity > id)
                && before.is_none_or(|id| node.identity < id)
            {
                visit(RunRange::Slot(
                    node.value.result.as_ref().expect("native leaf"),
                ));
            }
            walk(&node.right, after, before, visit, visits);
        }
        let mut visits = 0;
        walk(&self.root, after, before, visit, &mut visits);
        visits
    }
}

fn denial(cause: ReductionRunStop<LeaseDenial>) -> LeaseDenial {
    match cause {
        ReductionRunStop::Hook(cause) => cause,
        _ => LeaseDenial::charged_bytes_overflow(),
    }
}

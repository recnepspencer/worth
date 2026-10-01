use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PreparedDependencyCapture {
    pub(super) edges: Vec<PreparedDependencyEdge>,
}

impl PreparedDependencyCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn checked_capture_heap_bound(
        inputs: &crate::data::node::BoundedSignalInputs,
    ) -> Option<u64> {
        let fixed = inputs
            .as_slice()
            .len()
            .checked_mul(std::mem::size_of::<PreparedDependencyEdge>())?;
        u64::try_from(fixed)
            .ok()?
            .checked_add(inputs.captured_scope_heap_bound()?)
    }

    pub fn record(&mut self, source: NodeId, aspect: Aspect, scope: Option<PartitionSubscription>) {
        let edge = PreparedDependencyEdge {
            source,
            aspect,
            scope,
        };
        if let Some(last) = self.edges.last() {
            match compare_prepared_dependency_edges(last, &edge) {
                Ordering::Less => {
                    self.edges.push(edge);
                    return;
                }
                Ordering::Equal => return,
                Ordering::Greater => {}
            }
        }
        match self
            .edges
            .binary_search_by(|candidate| compare_prepared_dependency_edges(candidate, &edge))
        {
            Ok(_) => {}
            Err(index) => self.edges.insert(index, edge),
        }
    }

    pub fn as_slice(&self) -> &[PreparedDependencyEdge] {
        &self.edges
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn into_sorted_unique(self) -> Self {
        debug_assert!(is_sorted_unique(self.edges.as_slice()));
        self
    }

    pub fn canonicalize_unordered(mut self) -> Self {
        self.edges.sort_by(compare_prepared_dependency_edges);
        self.edges.dedup_by(|left, right| {
            compare_prepared_dependency_edges(left, right) == Ordering::Equal
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedDependencyEdge {
    pub source: NodeId,
    pub aspect: Aspect,
    #[serde(default)]
    pub scope: Option<PartitionSubscription>,
}

pub(crate) fn compare_prepared_dependency_edges(
    left: &PreparedDependencyEdge,
    right: &PreparedDependencyEdge,
) -> Ordering {
    (
        left.source.index(),
        left.source.generation(),
        left.aspect.index(),
        left.scope.as_ref(),
    )
        .cmp(&(
            right.source.index(),
            right.source.generation(),
            right.aspect.index(),
            right.scope.as_ref(),
        ))
}

fn is_sorted_unique(edges: &[PreparedDependencyEdge]) -> bool {
    edges.windows(2).all(|pair| {
        if let [left, right] = pair {
            compare_prepared_dependency_edges(left, right) == Ordering::Less
        } else {
            true
        }
    })
}

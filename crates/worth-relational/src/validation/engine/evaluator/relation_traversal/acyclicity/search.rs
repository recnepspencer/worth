//! One bounded visited/frontier extent, reused through all planned-edge searches.
use std::mem::size_of;

use crate::transactions::data::EntityReference;
use crate::validation::engine::context::InvariantExecutionContext;

use super::graph::TraversalGraph;

pub(super) struct ReachabilitySearch {
    visited: Vec<u8>,
    frontier: Vec<usize>,
}

impl ReachabilitySearch {
    pub(super) fn prepare(
        graph: &TraversalGraph<'_>,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Option<Self> {
        let count = graph.vertices.len();
        let bytes = graph.storage_bytes().checked_add(
            u64::try_from(count.checked_mul(size_of::<u8>() + size_of::<usize>())?).ok()?,
        )?;
        if !context.check_scratch_peak(bytes) || !context.checkpoint(count as u64) {
            return None;
        }
        let search = Self {
            visited: vec![0; count],
            frontier: Vec::with_capacity(count),
        };
        let actual = graph.storage_bytes().checked_add(
            (search.visited.capacity() * size_of::<u8>()
                + search.frontier.capacity() * size_of::<usize>()) as u64,
        )?;
        context.check_scratch_peak(actual).then_some(search)
    }

    /// O((V + E) log V) per search; repeated reachability remains actual work.
    pub(super) fn reaches(
        &mut self,
        graph: &TraversalGraph<'_>,
        start: &EntityReference,
        target: &EntityReference,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Option<bool> {
        if !context.checkpoint(self.visited.len() as u64) {
            return None;
        }
        self.visited.fill(0);
        self.frontier.clear();
        let start = graph.vertex_index(start, context)?;
        self.visited[start] = 1;
        self.frontier.push(start);
        while let Some(index) = self.frontier.pop() {
            if !context.checkpoint(1) {
                return None;
            }
            for &(_, next) in graph.successors(graph.vertices[index], context)? {
                if !context.checkpoint(2) {
                    return None;
                }
                if next == target {
                    return Some(true);
                }
                let next = graph.vertex_index(next, context)?;
                if self.visited[next] == 0 {
                    self.visited[next] = 1;
                    // Every vertex enters once, so the admitted capacity never grows.
                    self.frontier.push(next);
                }
            }
        }
        Some(false)
    }
}

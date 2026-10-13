//! Borrowed adjacency; no endpoint or authored key is copied into scratch.
use std::mem::size_of;

use crate::transactions::data::EntityReference;
use crate::validation::engine::context::InvariantExecutionContext;
use crate::validation::engine::request::PreparedRelationIntegrityScope;

pub(super) struct TraversalGraph<'scope> {
    edges: Vec<(&'scope EntityReference, &'scope EntityReference)>,
    pub(super) vertices: Vec<&'scope EntityReference>,
}

impl<'scope> TraversalGraph<'scope> {
    /// O(E log E) work and O(E) scratch for the exact prepared relation scope.
    /// Duplicate edges remain present and are charged on every traversal.
    pub(super) fn prepare(
        scope: &'scope PreparedRelationIntegrityScope,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Option<Self> {
        let mut edge_count = scope.planned_edges.len();
        for targets in scope.visible_successors.values() {
            if !context.checkpoint(1) {
                return None;
            }
            edge_count = edge_count
                .checked_add(targets.len())
                .expect("prepared edge storage has representable cardinality");
        }
        let vertex_capacity = edge_count
            .checked_mul(2)
            .expect("borrowed endpoints fit the prepared edge layout");
        // Both graph buffers coexist, including the pending vertex allocation.
        // After deduplication search admission adds its exact vertex count to
        // these retained capacities, before allocating visited/frontier storage.
        let bytes = scratch_bytes(edge_count, vertex_capacity)
            .expect("borrowed traversal storage fits the prepared edge layout");
        if !context.check_scratch_peak(bytes) {
            return None;
        }
        let mut graph = Self {
            edges: Vec::with_capacity(edge_count),
            vertices: Vec::with_capacity(vertex_capacity),
        };
        if !context.check_scratch_peak(graph.storage_bytes()) {
            return None;
        }
        let (edges, vertices) = (&mut graph.edges, &mut graph.vertices);
        for edge in &scope.planned_edges {
            if !context.checkpoint(1) {
                return None;
            }
            edges.push((&edge.source, &edge.target));
            vertices.extend([&edge.source, &edge.target]);
        }
        for (source, targets) in &scope.visible_successors {
            for target in targets {
                if !context.checkpoint(1) {
                    return None;
                }
                edges.push((source, target));
                vertices.extend([source, target]);
            }
        }
        if !super::index_order::order_index(edges, context)
            || !super::index_order::order_index(vertices, context)
            || !context.checkpoint(vertices.len() as u64)
        {
            return None;
        }
        vertices.dedup();
        Some(graph)
    }

    pub(super) fn storage_bytes(&self) -> u64 {
        (self.edges.capacity() * size_of::<(&EntityReference, &EntityReference)>()
            + self.vertices.capacity() * size_of::<&EntityReference>()) as u64
    }

    pub(super) fn vertex_index(
        &self,
        vertex: &EntityReference,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Option<usize> {
        let index =
            super::index_order::lower_bound(&self.vertices, context, |probe| *probe < vertex)?;
        assert_eq!(
            *self.vertices[index], *vertex,
            "prepared endpoint belongs to the index"
        );
        Some(index)
    }

    pub(super) fn successors<'graph>(
        &'graph self,
        source: &EntityReference,
        context: &InvariantExecutionContext<'_, '_>,
    ) -> Option<&'graph [(&'scope EntityReference, &'scope EntityReference)]> {
        let first =
            super::index_order::lower_bound(&self.edges, context, |(from, _)| *from < source)?;
        let end =
            super::index_order::lower_bound(&self.edges, context, |(from, _)| *from <= source)?;
        Some(&self.edges[first..end])
    }
}

fn scratch_bytes(edges: usize, vertices: usize) -> Option<u64> {
    edges
        .checked_mul(size_of::<(&EntityReference, &EntityReference)>())?
        .checked_add(vertices.checked_mul(size_of::<&EntityReference>())?)?
        .try_into()
        .ok()
}

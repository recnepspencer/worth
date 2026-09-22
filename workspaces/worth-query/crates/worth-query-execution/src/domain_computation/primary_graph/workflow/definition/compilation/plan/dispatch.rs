use super::CompiledWorkflowSemanticConnection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CompiledWorkflowDispatchEdge {
    pub(super) connection: usize,
    pub(super) peer: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(in super::super) struct CompiledWorkflowDispatch {
    outgoing: Box<[Box<[CompiledWorkflowDispatchEdge]>]>,
    incoming: Box<[Box<[CompiledWorkflowDispatchEdge]>]>,
}

impl CompiledWorkflowDispatch {
    pub(in super::super) fn build(
        node_count: usize,
        connections: &[CompiledWorkflowSemanticConnection],
    ) -> Self {
        let mut outgoing = vec![Vec::new(); node_count];
        let mut incoming = vec![Vec::new(); node_count];
        for (connection, meaning) in connections.iter().enumerate() {
            outgoing[meaning.source].push(CompiledWorkflowDispatchEdge {
                connection,
                peer: meaning.target,
            });
            incoming[meaning.target].push(CompiledWorkflowDispatchEdge {
                connection,
                peer: meaning.source,
            });
        }
        Self {
            outgoing: outgoing
                .into_iter()
                .map(Vec::into_boxed_slice)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            incoming: incoming
                .into_iter()
                .map(Vec::into_boxed_slice)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub(super) fn outgoing(&self, node: usize) -> &[CompiledWorkflowDispatchEdge] {
        &self.outgoing[node]
    }

    pub(super) fn incoming(&self, node: usize) -> &[CompiledWorkflowDispatchEdge] {
        &self.incoming[node]
    }

    pub(in super::super) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(std::mem::size_of_val(self.outgoing.as_ref()))
            .saturating_add(std::mem::size_of_val(self.incoming.as_ref()))
            .saturating_add(
                self.outgoing
                    .iter()
                    .chain(self.incoming.iter())
                    .map(|edges| std::mem::size_of_val(edges.as_ref()))
                    .sum::<usize>(),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;

    use super::*;
    use crate::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowConnectionKind;

    #[test]
    fn sparse_local_dispatch_does_not_include_unrelated_connections() {
        const NODE_COUNT: usize = 10_000;
        let connections = (0..NODE_COUNT - 1)
            .map(|source| CompiledWorkflowSemanticConnection {
                source,
                target: source + 1,
                kind: Arc::new(CompiledWorkflowConnectionKind::Control(
                    ApplicationWorkflowControlOutcome::Completed,
                )),
            })
            .collect::<Vec<_>>();

        let dispatch = CompiledWorkflowDispatch::build(NODE_COUNT, &connections);

        assert_eq!(dispatch.outgoing(5_000).len(), 1);
        assert_eq!(dispatch.outgoing(5_000)[0].connection, 5_000);
        assert_eq!(dispatch.outgoing(5_000)[0].peer, 5_001);
        assert_eq!(dispatch.incoming(5_000).len(), 1);
        assert_eq!(dispatch.incoming(5_000)[0].connection, 4_999);
        assert_eq!(dispatch.incoming(5_000)[0].peer, 4_999);
    }
}

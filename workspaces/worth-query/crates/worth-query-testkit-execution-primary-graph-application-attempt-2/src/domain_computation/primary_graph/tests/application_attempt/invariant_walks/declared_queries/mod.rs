//! Real installed declared-query closure courts over the existing authored graph.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::declarations::NodeRow;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::declared_queries::declarations::Parameters;

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::Graph;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::World;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotResult;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
pub fn diamond(nodes: usize, blocked: Vec<u64>) -> World {
    crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::install(Graph {
        nodes,
        edges: vec![(0, 1), (0, 2), (1, 3), (2, 3), (3, 3), (3, 4), (4, 1)],
        active: false,
        blocked,
    })
}
pub fn keys<Q>(result: &WorthQueryApplicationOneShotResult<Q, NodeRow>) -> Vec<u64> {
    let mut values: Vec<_> = result.rows().iter().map(|row| row.key).collect();
    values.sort();
    values
}

#[path = "read/mod.rs"]
pub mod read;

//! Real installed declared-query closure courts over the existing authored graph.
pub(in crate::domain_computation::primary_graph) mod declarations;
pub(in crate::domain_computation::primary_graph) mod read;
mod resources;
mod topology;
use super::{
    schema::*,
    world::{self, Graph, World},
};
use crate::domain_computation::primary_graph::*;
use declarations::*;

fn diamond(nodes: usize, blocked: Vec<u64>) -> World {
    world::install(Graph {
        nodes,
        edges: vec![(0, 1), (0, 2), (1, 3), (2, 3), (3, 3), (3, 4), (4, 1)],
        active: false,
        blocked,
    })
}
fn keys<Q>(result: &WorthQueryApplicationOneShotResult<Q, NodeRow>) -> Vec<u64> {
    let mut values: Vec<_> = result.rows().iter().map(|row| row.key).collect();
    values.sort();
    values
}

mod currentness;

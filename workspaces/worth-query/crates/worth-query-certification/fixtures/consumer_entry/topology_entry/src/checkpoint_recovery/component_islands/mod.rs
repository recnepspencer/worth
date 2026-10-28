//! Public topology edits checked against independently authored undirected edges.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::AggregateLineageKind;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
mod differential;
mod identity_model;
mod locality;
mod union_find;
mod world;
type Member = WorthQueryInvariantEntityIdentity<CheckpointSchema, Body>;
type Name = WorthQueryAggregateIdentity<CheckpointSchema, PlanarIslands>;
type Event = (AggregateLineageKind, [u8; 32], [u8; 32]);

#[derive(Debug)]
struct Group {
    name: Name,
    members: BTreeSet<Member>,
}
struct History {
    events: Vec<Event>,
    latest: Option<[u8; 32]>,
    retired: bool,
}
fn key(ring: usize, vertex: &str) -> String {
    format!("ring-{ring}-{vertex}")
}
fn keys(rings: usize) -> Vec<String> {
    (0..rings)
        .flat_map(|ring| ["a", "b", "c"].map(|v| key(ring, v)))
        .collect()
}

//! Active native ownership court after the real MODE2 Query publication.
use crate::domain_computation::primary_graph;
use crate::facade as runtime_api;
use crate::facade::primary_graph::*;
use fixture_values::PositiveLength;
use runtime_api::application_contribution::*;
use worth_query_installation::{
    facade as runtime_domain, worth_query_conditional_node as conditional_node,
};
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/base_model.rs"]
mod base_model;
use base_model::*;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/principal.rs"]
mod principal;
use principal::*;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/mutation/output_contract.rs"]
mod output_contract;
use output_contract::*;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_topology.rs"]
mod planar_topology;
use planar_topology::*;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_invariant.rs"]
mod planar_invariant;

#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/planar_read.rs"]
mod planar_read;
use planar_read::*;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/region_output.rs"]
mod region_output;
use region_output::*;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/output_producer.rs"]
mod output_producer;
use output_producer::*;
mod facts {
    use super::*;
    include!("../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/facts/declaration.rs");
}
mod readiness {
    include!("../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/readiness/source.rs");
}
include!("../../../../../../worth-query-certification/fixtures/consumer_entry/topology_entry/src/producer/resources.rs");
mod admission;
mod birth;
mod fixture_values;
mod installation;
mod journey;

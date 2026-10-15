//! Consolidated integration suite for Query audience and source fences.

mod query_audience_fixture;
mod query_source_fixture;

#[path = "query_contracts/audience.rs"]
mod audience;
#[path = "query_contracts/consumer_docs.rs"]
mod consumer_docs;
#[path = "query_contracts/internal_authority_topology.rs"]
mod internal_authority_topology;
#[path = "query_contracts/macro_equivalence.rs"]
mod macro_equivalence;
#[path = "query_contracts/public_reexport.rs"]
mod public_reexport;
#[path = "query_contracts/public_signature.rs"]
mod public_signature;
#[path = "query_contracts/source_audience.rs"]
mod source_audience;
#[path = "query_contracts/source_path.rs"]
mod source_path;
#[path = "query_contracts/topology.rs"]
mod topology;

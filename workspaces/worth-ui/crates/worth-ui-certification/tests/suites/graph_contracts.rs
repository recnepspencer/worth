#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Semantic integration suite. Individual responsibilities remain in named child modules.

#[path = "../graph_aspect_index_runtime.rs"]
mod graph_aspect_index_runtime;
#[path = "../graph_authority_runtime.rs"]
mod graph_authority_runtime;
#[path = "../graph_closeout_runtime.rs"]
mod graph_closeout_runtime;
#[path = "../graph_evidence_lookup_runtime.rs"]
mod graph_evidence_lookup_runtime;
#[path = "../graph_inspection_runtime.rs"]
mod graph_inspection_runtime;
#[path = "../graph_instantiation_runtime.rs"]
mod graph_instantiation_runtime;
#[path = "../graph_lookup_runtime.rs"]
mod graph_lookup_runtime;
#[path = "../graph_mutation_runtime.rs"]
mod graph_mutation_runtime;
#[path = "../graph_participation_runtime.rs"]
mod graph_participation_runtime;
#[path = "../graph_touch_origin_authority_runtime.rs"]
mod graph_touch_origin_authority_runtime;
#[path = "../graph_touch_runtime.rs"]
mod graph_touch_runtime;
#[path = "../graph_touch_runtime_origin_receipts.rs"]
mod graph_touch_runtime_origin_receipts;

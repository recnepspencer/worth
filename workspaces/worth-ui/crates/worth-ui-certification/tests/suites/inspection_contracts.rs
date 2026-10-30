#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Semantic integration suite. Individual responsibilities remain in named child modules.

#[path = "../ai_inspection_runtime.rs"]
mod ai_inspection_runtime;
#[path = "../aspect_evidence_lookup_runtime.rs"]
mod aspect_evidence_lookup_runtime;
#[path = "../aspect_evidence_slice_runtime.rs"]
mod aspect_evidence_slice_runtime;
#[path = "../evidence_materialization_runtime.rs"]
mod evidence_materialization_runtime;
#[path = "../evidence_reference_runtime.rs"]
mod evidence_reference_runtime;
#[path = "../evidence_slice_runtime.rs"]
mod evidence_slice_runtime;
#[path = "../inspection_boundary_runtime.rs"]
mod inspection_boundary_runtime;
#[path = "../inspection_closeout_runtime.rs"]
mod inspection_closeout_runtime;
#[path = "../inspection_cost_runtime.rs"]
mod inspection_cost_runtime;
#[path = "../mount_eligibility_runtime.rs"]
mod mount_eligibility_runtime;

#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Semantic integration suite. Individual responsibilities remain in named child modules.

#[path = "../declaration_aspect_contract_runtime.rs"]
mod declaration_aspect_contract_runtime;
#[path = "../declaration_authority_runtime.rs"]
mod declaration_authority_runtime;
#[path = "../declaration_closeout_runtime.rs"]
mod declaration_closeout_runtime;
#[path = "../declaration_declared_posture_runtime.rs"]
mod declaration_declared_posture_runtime;
#[path = "../declaration_evidence_lookup_runtime.rs"]
mod declaration_evidence_lookup_runtime;
#[path = "../declaration_family_admission_runtime.rs"]
mod declaration_family_admission_runtime;
#[path = "../declaration_graph_handoff_runtime.rs"]
mod declaration_graph_handoff_runtime;
#[path = "../declaration_structural_semantics_runtime.rs"]
mod declaration_structural_semantics_runtime;
#[path = "../declaration_support_snapshot_runtime.rs"]
mod declaration_support_snapshot_runtime;
#[path = "../declaration_contracts/projection_declaration/mod.rs"]
mod projection_declaration;

//! `borrowed_canonical_material` keeps its own test process: it constructs process-wide
//! state that the shared suite binary cannot host twice.

#[path = "../borrowed_canonical_material.rs"]
mod borrowed_canonical_material;

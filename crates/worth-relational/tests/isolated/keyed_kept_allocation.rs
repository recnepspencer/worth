//! `keyed_kept_allocation` keeps its own test process: it constructs process-wide
//! state that the shared suite binary cannot host twice.

#[path = "../keyed_kept_allocation.rs"]
mod keyed_kept_allocation;

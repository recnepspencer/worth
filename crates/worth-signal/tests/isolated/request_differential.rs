//! `request_differential` keeps its own test process: it constructs process-wide
//! state that the shared suite binary cannot host twice.

#[path = "../request_differential.rs"]
mod request_differential;

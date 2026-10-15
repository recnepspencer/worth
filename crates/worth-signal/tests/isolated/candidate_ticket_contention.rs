//! `candidate_ticket_contention` keeps its own test process: it constructs process-wide
//! state that the shared suite binary cannot host twice.

#[path = "../candidate_ticket_contention.rs"]
mod candidate_ticket_contention;

//! Serial integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "serial/execution_request_denials.rs"]
pub(crate) mod execution_request_denials;

#[path = "serial/candidate_ticket_contention.rs"]
// `candidate_ticket_contention` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod candidate_ticket_contention;

#[path = "serial/request_differential.rs"]
// `request_differential` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod request_differential;

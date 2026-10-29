//! Exhaustive integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "exhaustive/signal_owner_services.rs"]
pub(crate) mod signal_owner_services;

#[path = "exhaustive/request_differential.rs"]
// `request_differential` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod request_differential;

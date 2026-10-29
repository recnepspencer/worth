//! Serial integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "serial/keyed_kept_allocation.rs"]
// `keyed_kept_allocation` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod keyed_kept_allocation;

#[path = "serial/leased_preparation.rs"]
// `leased_preparation` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod leased_preparation;

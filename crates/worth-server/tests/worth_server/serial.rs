//! Serial integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "serial/product_operation_phase_thirteen_shared_read.rs"]
// `product_operation_phase_thirteen_shared_read` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod product_operation_phase_thirteen_shared_read;

#[path = "serial/product_read_nested_lease.rs"]
// `product_read_nested_lease` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod product_read_nested_lease;

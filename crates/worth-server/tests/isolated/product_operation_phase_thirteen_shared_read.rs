//! `product_operation_phase_thirteen_shared_read` keeps its own test process: it constructs process-wide
//! state that the shared suite binary cannot host twice.

#[allow(dead_code, unused_imports)]
#[path = "../support/product_adapter_phase_nine/fixture.rs"]
pub mod shared_support_product_adapter_phase_nine_fixture;
#[allow(dead_code, unused_imports)]
#[path = "../support/product_operation_phase_thirteen/fixture.rs"]
pub mod shared_support_product_operation_phase_thirteen_fixture;
#[allow(dead_code, unused_imports)]
#[path = "../support/product_result/schema_bound_json.rs"]
pub mod shared_support_product_result_schema_bound_json;
#[allow(dead_code, unused_imports)]
#[path = "../support/query_handoff/runtime.rs"]
pub mod shared_support_query_handoff_runtime;
#[allow(dead_code, unused_imports)]
#[path = "../support/test_bootstrap_policy.rs"]
pub mod shared_support_test_bootstrap_policy;
#[path = "../product_operation_phase_thirteen_shared_read.rs"]
mod product_operation_phase_thirteen_shared_read;

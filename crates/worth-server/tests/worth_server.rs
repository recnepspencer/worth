//! Integration courts grouped by execution cost.

#[path = "worth_server/fast.rs"]
mod fast;

#[path = "worth_server/exhaustive.rs"]
mod exhaustive;

#[path = "worth_server/compile_fail.rs"]
mod compile_fail;

#[path = "worth_server/serial.rs"]
mod serial;

#[path = "worth_server/fixtures/support/compat_http/phase_eight/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_eight_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_eight/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_eight_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_four_assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_four_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_four_runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_four_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_nine/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_nine_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_nine/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_nine_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_seven/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_seven_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_seven/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_seven_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_six_runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_six_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_ten/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_ten_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_ten/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_ten_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_thirteen/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_thirteen_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_thirteen/bundle.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_thirteen_bundle;

#[path = "worth_server/fixtures/support/compat_http/phase_thirteen/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_thirteen_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_three_runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_three_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_twelve/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_twelve_assertions;

#[path = "worth_server/fixtures/support/compat_http/phase_twelve/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_twelve_runtime;

#[path = "worth_server/fixtures/support/compat_http/phase_two_runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_compat_http_phase_two_runtime;

#[path = "worth_server/fixtures/support/direct_context_runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_direct_context_runtime;

#[path = "worth_server/fixtures/support/durable_product_mutation/mod.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_durable_product_mutation;

#[path = "worth_server/fixtures/support/operation_request/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_operation_request_runtime;

#[path = "worth_server/fixtures/support/product_adapter_phase_nine/fixture.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_product_adapter_phase_nine_fixture;

#[path = "worth_server/fixtures/support/product_operation_phase_eleven/backend.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_product_operation_phase_eleven_backend;

#[path = "worth_server/fixtures/support/product_operation_phase_thirteen/fixture.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_product_operation_phase_thirteen_fixture;

#[path = "worth_server/fixtures/support/product_operation_phase_thirteen/parity_driver.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_product_operation_phase_thirteen_parity_driver;

#[path = "worth_server/fixtures/support/product_result/schema_bound_json.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_product_result_schema_bound_json;

#[path = "worth_server/fixtures/support/product_session_phase_ten/fixture.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_product_session_phase_ten_fixture;

#[path = "worth_server/fixtures/support/query_handoff/fixture.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_query_handoff_fixture;

#[path = "worth_server/fixtures/support/query_handoff/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_query_handoff_runtime;

#[path = "worth_server/fixtures/support/response_fixture.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_response_fixture;

#[path = "worth_server/fixtures/support/route_assembly_phase_twelve/fixture.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_route_assembly_phase_twelve_fixture;

#[path = "worth_server/fixtures/support/route_assembly_phase_twelve/request_driver.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_route_assembly_phase_twelve_request_driver;

#[path = "worth_server/fixtures/support/test_bootstrap_policy.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_test_bootstrap_policy;

#[path = "worth_server/fixtures/support/worth_native/assertions.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_worth_native_assertions;

#[path = "worth_server/fixtures/support/worth_native/runtime.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support_worth_native_runtime;

#[path = "worth_server/test_process.rs"]
mod test_process;

//! The server_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_eight/assertions.rs"]
pub mod shared_support_compat_http_phase_eight_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_eight/runtime.rs"]
pub mod shared_support_compat_http_phase_eight_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_four_assertions.rs"]
pub mod shared_support_compat_http_phase_four_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_four_runtime.rs"]
pub mod shared_support_compat_http_phase_four_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_nine/assertions.rs"]
pub mod shared_support_compat_http_phase_nine_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_nine/runtime.rs"]
pub mod shared_support_compat_http_phase_nine_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_seven/assertions.rs"]
pub mod shared_support_compat_http_phase_seven_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_seven/runtime.rs"]
pub mod shared_support_compat_http_phase_seven_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_six_runtime.rs"]
pub mod shared_support_compat_http_phase_six_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_ten/assertions.rs"]
pub mod shared_support_compat_http_phase_ten_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_ten/runtime.rs"]
pub mod shared_support_compat_http_phase_ten_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_thirteen/assertions.rs"]
pub mod shared_support_compat_http_phase_thirteen_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_thirteen/bundle.rs"]
pub mod shared_support_compat_http_phase_thirteen_bundle;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_thirteen/runtime.rs"]
pub mod shared_support_compat_http_phase_thirteen_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_three_runtime.rs"]
pub mod shared_support_compat_http_phase_three_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_twelve/assertions.rs"]
pub mod shared_support_compat_http_phase_twelve_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_twelve/runtime.rs"]
pub mod shared_support_compat_http_phase_twelve_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/compat_http/phase_two_runtime.rs"]
pub mod shared_support_compat_http_phase_two_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/direct_context_runtime.rs"]
pub mod shared_support_direct_context_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/durable_product_mutation/mod.rs"]
pub mod shared_support_durable_product_mutation;
#[allow(dead_code, unused_imports)]
#[path = "support/operation_request/runtime.rs"]
pub mod shared_support_operation_request_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/product_adapter_phase_nine/fixture.rs"]
pub mod shared_support_product_adapter_phase_nine_fixture;
#[allow(dead_code, unused_imports)]
#[path = "support/product_operation_phase_eleven/backend.rs"]
pub mod shared_support_product_operation_phase_eleven_backend;
#[allow(dead_code, unused_imports)]
#[path = "support/product_operation_phase_thirteen/fixture.rs"]
pub mod shared_support_product_operation_phase_thirteen_fixture;
#[allow(dead_code, unused_imports)]
#[path = "support/product_operation_phase_thirteen/parity_driver.rs"]
pub mod shared_support_product_operation_phase_thirteen_parity_driver;
#[allow(dead_code, unused_imports)]
#[path = "support/product_result/schema_bound_json.rs"]
pub mod shared_support_product_result_schema_bound_json;
#[allow(dead_code, unused_imports)]
#[path = "support/product_session_phase_ten/fixture.rs"]
pub mod shared_support_product_session_phase_ten_fixture;
#[allow(dead_code, unused_imports)]
#[path = "support/query_handoff/fixture.rs"]
pub mod shared_support_query_handoff_fixture;
#[allow(dead_code, unused_imports)]
#[path = "support/query_handoff/runtime.rs"]
pub mod shared_support_query_handoff_runtime;
#[allow(dead_code, unused_imports)]
#[path = "support/response_fixture.rs"]
pub mod shared_support_response_fixture;
#[allow(dead_code, unused_imports)]
#[path = "support/route_assembly_phase_twelve/fixture.rs"]
pub mod shared_support_route_assembly_phase_twelve_fixture;
#[allow(dead_code, unused_imports)]
#[path = "support/route_assembly_phase_twelve/request_driver.rs"]
pub mod shared_support_route_assembly_phase_twelve_request_driver;
#[allow(dead_code, unused_imports)]
#[path = "support/test_bootstrap_policy.rs"]
pub mod shared_support_test_bootstrap_policy;
#[allow(dead_code, unused_imports)]
#[path = "support/worth_native/assertions.rs"]
pub mod shared_support_worth_native_assertions;
#[allow(dead_code, unused_imports)]
#[path = "support/worth_native/runtime.rs"]
pub mod shared_support_worth_native_runtime;
#[path = "certification_suite.rs"]
mod certification_suite;
#[path = "compat_http_admission.rs"]
mod compat_http_admission;
#[path = "compat_http_entry.rs"]
mod compat_http_entry;
#[path = "compat_http_phase_eight.rs"]
mod compat_http_phase_eight;
#[path = "compat_http_phase_eight_boundary.rs"]
mod compat_http_phase_eight_boundary;
#[path = "compat_http_phase_eleven.rs"]
mod compat_http_phase_eleven;
#[path = "compat_http_phase_eleven_boundary.rs"]
mod compat_http_phase_eleven_boundary;
#[path = "compat_http_phase_five.rs"]
mod compat_http_phase_five;
#[path = "compat_http_phase_four.rs"]
mod compat_http_phase_four;
#[path = "compat_http_phase_nine.rs"]
mod compat_http_phase_nine;
#[path = "compat_http_phase_nine_boundary.rs"]
mod compat_http_phase_nine_boundary;
#[path = "compat_http_phase_seven.rs"]
mod compat_http_phase_seven;
#[path = "compat_http_phase_seven_boundary.rs"]
mod compat_http_phase_seven_boundary;
#[path = "compat_http_phase_six.rs"]
mod compat_http_phase_six;
#[path = "compat_http_phase_ten.rs"]
mod compat_http_phase_ten;
#[path = "compat_http_phase_ten_boundary.rs"]
mod compat_http_phase_ten_boundary;
#[path = "compat_http_phase_thirteen.rs"]
mod compat_http_phase_thirteen;
#[path = "compat_http_phase_thirteen_boundary.rs"]
mod compat_http_phase_thirteen_boundary;
#[path = "compat_http_phase_three.rs"]
mod compat_http_phase_three;
#[path = "compat_http_phase_three_followups.rs"]
mod compat_http_phase_three_followups;
#[path = "compat_http_phase_twelve.rs"]
mod compat_http_phase_twelve;
#[path = "compat_http_phase_twelve_boundary.rs"]
mod compat_http_phase_twelve_boundary;
#[path = "compat_http_phase_two.rs"]
mod compat_http_phase_two;
#[path = "compile_fail.rs"]
mod compile_fail;
#[path = "durable_product_mutation_concurrency.rs"]
mod durable_product_mutation_concurrency;
#[path = "durable_product_mutation_crash_recovery.rs"]
mod durable_product_mutation_crash_recovery;
#[path = "durable_product_mutation_idempotency.rs"]
mod durable_product_mutation_idempotency;
#[path = "durable_product_mutation_preconditions.rs"]
mod durable_product_mutation_preconditions;
#[path = "durable_product_mutation_specimens.rs"]
mod durable_product_mutation_specimens;
#[path = "external_resource_execution.rs"]
mod external_resource_execution;
#[path = "facade_bootstrap.rs"]
mod facade_bootstrap;
#[path = "middleware_progression.rs"]
mod middleware_progression;
#[path = "operation_admission.rs"]
mod operation_admission;
#[path = "operation_planning.rs"]
mod operation_planning;
#[path = "operation_readiness.rs"]
mod operation_readiness;
#[path = "operation_readiness_boundaries.rs"]
mod operation_readiness_boundaries;
#[path = "operation_readiness_preconditions.rs"]
mod operation_readiness_preconditions;
#[path = "operation_registry.rs"]
mod operation_registry;
#[path = "operation_request.rs"]
mod operation_request;
#[path = "operation_request_boundaries.rs"]
mod operation_request_boundaries;
#[path = "operation_scheduler.rs"]
mod operation_scheduler;
#[path = "operation_scheduler_phase_eight.rs"]
mod operation_scheduler_phase_eight;
#[path = "product_adapter_phase_nine.rs"]
mod product_adapter_phase_nine;
#[path = "product_adapter_phase_nine_query_derived.rs"]
mod product_adapter_phase_nine_query_derived;
#[path = "product_adapter_registration_contracts.rs"]
mod product_adapter_registration_contracts;
#[path = "product_operation_application_authorization.rs"]
mod product_operation_application_authorization;
#[path = "product_operation_phase_eleven.rs"]
mod product_operation_phase_eleven;
#[path = "product_operation_phase_eleven_boundaries.rs"]
mod product_operation_phase_eleven_boundaries;
#[path = "product_operation_phase_eleven_stateful.rs"]
mod product_operation_phase_eleven_stateful;
#[path = "product_operation_phase_thirteen.rs"]
mod product_operation_phase_thirteen;
#[path = "product_operation_phase_thirteen_certification.rs"]
mod product_operation_phase_thirteen_certification;
#[path = "product_operation_phase_thirteen_parity.rs"]
mod product_operation_phase_thirteen_parity;
#[path = "product_result_artifact.rs"]
mod product_result_artifact;
#[path = "product_result_registration.rs"]
mod product_result_registration;
#[path = "product_result_surface_parity.rs"]
mod product_result_surface_parity;
#[path = "product_session_phase_ten.rs"]
mod product_session_phase_ten;
#[path = "product_session_scope_phase_ten.rs"]
mod product_session_scope_phase_ten;
#[path = "query_dependency_audit.rs"]
mod query_dependency_audit;
#[path = "query_handoff_denials.rs"]
mod query_handoff_denials;
#[path = "query_handoff_parity.rs"]
mod query_handoff_parity;
#[path = "request_context_resolution.rs"]
mod request_context_resolution;
#[path = "route_assembly_phase_twelve.rs"]
mod route_assembly_phase_twelve;
#[path = "route_assembly_phase_twelve_boundary.rs"]
mod route_assembly_phase_twelve_boundary;
#[path = "surface_family_isolation.rs"]
mod surface_family_isolation;
#[path = "surface_registration.rs"]
mod surface_registration;
#[path = "worth_native_facade_entry.rs"]
mod worth_native_facade_entry;

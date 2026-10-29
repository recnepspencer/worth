//! Fast integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "fast/certification_suite.rs"]
// The server_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod certification_suite;

#[path = "fast/compat_http_admission.rs"]
pub(crate) mod compat_http_admission;

#[path = "fast/compat_http_entry.rs"]
pub(crate) mod compat_http_entry;

#[path = "fast/compat_http_phase_eight.rs"]
pub(crate) mod compat_http_phase_eight;

#[path = "fast/compat_http_phase_eight_boundary.rs"]
pub(crate) mod compat_http_phase_eight_boundary;

#[path = "fast/compat_http_phase_eleven.rs"]
pub(crate) mod compat_http_phase_eleven;

#[path = "fast/compat_http_phase_eleven_boundary.rs"]
pub(crate) mod compat_http_phase_eleven_boundary;

#[path = "fast/compat_http_phase_five.rs"]
pub(crate) mod compat_http_phase_five;

#[path = "fast/compat_http_phase_four.rs"]
pub(crate) mod compat_http_phase_four;

#[path = "fast/compat_http_phase_nine.rs"]
pub(crate) mod compat_http_phase_nine;

#[path = "fast/compat_http_phase_nine_boundary.rs"]
pub(crate) mod compat_http_phase_nine_boundary;

#[path = "fast/compat_http_phase_seven.rs"]
pub(crate) mod compat_http_phase_seven;

#[path = "fast/compat_http_phase_seven_boundary.rs"]
pub(crate) mod compat_http_phase_seven_boundary;

#[path = "fast/compat_http_phase_six.rs"]
pub(crate) mod compat_http_phase_six;

#[path = "fast/compat_http_phase_ten.rs"]
pub(crate) mod compat_http_phase_ten;

#[path = "fast/compat_http_phase_ten_boundary.rs"]
pub(crate) mod compat_http_phase_ten_boundary;

#[path = "fast/compat_http_phase_thirteen.rs"]
pub(crate) mod compat_http_phase_thirteen;

#[path = "fast/compat_http_phase_thirteen_boundary.rs"]
pub(crate) mod compat_http_phase_thirteen_boundary;

#[path = "fast/compat_http_phase_three.rs"]
pub(crate) mod compat_http_phase_three;

#[path = "fast/compat_http_phase_three_followups.rs"]
pub(crate) mod compat_http_phase_three_followups;

#[path = "fast/compat_http_phase_twelve.rs"]
pub(crate) mod compat_http_phase_twelve;

#[path = "fast/compat_http_phase_twelve_boundary.rs"]
pub(crate) mod compat_http_phase_twelve_boundary;

#[path = "fast/compat_http_phase_two.rs"]
pub(crate) mod compat_http_phase_two;

#[path = "fast/durable_product_mutation_concurrency.rs"]
pub(crate) mod durable_product_mutation_concurrency;

#[path = "fast/durable_product_mutation_crash_recovery.rs"]
pub(crate) mod durable_product_mutation_crash_recovery;

#[path = "fast/durable_product_mutation_idempotency.rs"]
pub(crate) mod durable_product_mutation_idempotency;

#[path = "fast/durable_product_mutation_preconditions.rs"]
pub(crate) mod durable_product_mutation_preconditions;

#[path = "fast/durable_product_mutation_specimens.rs"]
pub(crate) mod durable_product_mutation_specimens;

#[path = "fast/external_resource_execution.rs"]
pub(crate) mod external_resource_execution;

#[path = "fast/facade_bootstrap.rs"]
pub(crate) mod facade_bootstrap;

#[path = "fast/middleware_progression.rs"]
pub(crate) mod middleware_progression;

#[path = "fast/operation_admission.rs"]
pub(crate) mod operation_admission;

#[path = "fast/operation_planning.rs"]
pub(crate) mod operation_planning;

#[path = "fast/operation_readiness.rs"]
pub(crate) mod operation_readiness;

#[path = "fast/operation_readiness_boundaries.rs"]
pub(crate) mod operation_readiness_boundaries;

#[path = "fast/operation_readiness_preconditions.rs"]
pub(crate) mod operation_readiness_preconditions;

#[path = "fast/operation_registry.rs"]
pub(crate) mod operation_registry;

#[path = "fast/operation_request.rs"]
pub(crate) mod operation_request;

#[path = "fast/operation_request_boundaries.rs"]
pub(crate) mod operation_request_boundaries;

#[path = "fast/operation_scheduler.rs"]
pub(crate) mod operation_scheduler;

#[path = "fast/operation_scheduler_phase_eight.rs"]
pub(crate) mod operation_scheduler_phase_eight;

#[path = "fast/product_adapter_phase_nine.rs"]
pub(crate) mod product_adapter_phase_nine;

#[path = "fast/product_adapter_phase_nine_query_derived.rs"]
pub(crate) mod product_adapter_phase_nine_query_derived;

#[path = "fast/product_adapter_registration_contracts.rs"]
pub(crate) mod product_adapter_registration_contracts;

#[path = "fast/product_operation_application_authorization.rs"]
pub(crate) mod product_operation_application_authorization;

#[path = "fast/product_operation_phase_eleven.rs"]
pub(crate) mod product_operation_phase_eleven;

#[path = "fast/product_operation_phase_eleven_boundaries.rs"]
pub(crate) mod product_operation_phase_eleven_boundaries;

#[path = "fast/product_operation_phase_eleven_stateful.rs"]
pub(crate) mod product_operation_phase_eleven_stateful;

#[path = "fast/product_operation_phase_thirteen.rs"]
pub(crate) mod product_operation_phase_thirteen;

#[path = "fast/product_operation_phase_thirteen_certification.rs"]
pub(crate) mod product_operation_phase_thirteen_certification;

#[path = "fast/product_operation_phase_thirteen_parity.rs"]
pub(crate) mod product_operation_phase_thirteen_parity;

#[path = "fast/product_result_artifact.rs"]
pub(crate) mod product_result_artifact;

#[path = "fast/product_result_registration.rs"]
pub(crate) mod product_result_registration;

#[path = "fast/product_result_surface_parity.rs"]
pub(crate) mod product_result_surface_parity;

#[path = "fast/product_session_phase_ten.rs"]
pub(crate) mod product_session_phase_ten;

#[path = "fast/product_session_scope_phase_ten.rs"]
pub(crate) mod product_session_scope_phase_ten;

#[path = "fast/query_dependency_audit.rs"]
pub(crate) mod query_dependency_audit;

#[path = "fast/query_handoff_denials.rs"]
pub(crate) mod query_handoff_denials;

#[path = "fast/query_handoff_parity.rs"]
pub(crate) mod query_handoff_parity;

#[path = "fast/request_context_resolution.rs"]
pub(crate) mod request_context_resolution;

#[path = "fast/route_assembly_phase_twelve.rs"]
pub(crate) mod route_assembly_phase_twelve;

#[path = "fast/route_assembly_phase_twelve_boundary.rs"]
pub(crate) mod route_assembly_phase_twelve_boundary;

#[path = "fast/surface_family_isolation.rs"]
pub(crate) mod surface_family_isolation;

#[path = "fast/surface_registration.rs"]
pub(crate) mod surface_registration;

#[path = "fast/worth_native_facade_entry.rs"]
pub(crate) mod worth_native_facade_entry;

#[path = "fast/product_operation_phase_thirteen_shared_read.rs"]
// `product_operation_phase_thirteen_shared_read` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod product_operation_phase_thirteen_shared_read;

#[path = "fast/product_retry_ownership.rs"]
pub(crate) mod product_retry_ownership;

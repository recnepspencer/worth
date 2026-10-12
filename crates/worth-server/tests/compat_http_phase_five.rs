#[path = "support/compat_http/phase_five_runtime.rs"]
mod compat_http_phase_five_runtime;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;
#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;

#[path = "compat_http_phase_five/early_admission.rs"]
mod early_admission;
#[path = "compat_http_phase_five/malformed_uploads.rs"]
mod malformed_uploads;
#[path = "compat_http_phase_five/metadata_truth.rs"]
mod metadata_truth;
#[path = "compat_http_phase_five/transport_truth.rs"]
mod transport_truth;
#[path = "compat_http_phase_five/upload_input.rs"]
mod upload_input;

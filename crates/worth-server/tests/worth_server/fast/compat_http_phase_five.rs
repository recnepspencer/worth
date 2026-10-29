#[path = "../fixtures/support/compat_http/phase_five_runtime.rs"]
pub(crate) mod compat_http_phase_five_runtime;

#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;
#[path = "compat_http_phase_five/early_admission.rs"]
pub(crate) mod early_admission;

#[path = "compat_http_phase_five/malformed_uploads.rs"]
pub(crate) mod malformed_uploads;

#[path = "compat_http_phase_five/metadata_truth.rs"]
pub(crate) mod metadata_truth;

#[path = "compat_http_phase_five/transport_truth.rs"]
pub(crate) mod transport_truth;

#[path = "../fixtures/compat_http_phase_five/upload_input.rs"]
pub(crate) mod upload_input;

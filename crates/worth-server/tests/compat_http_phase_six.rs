#[path = "support/compat_http/phase_six_assertions.rs"]
mod compat_http_phase_six_assertions;
#[allow(unused_imports)]
use crate::shared_support_compat_http_phase_six_runtime as compat_http_phase_six_runtime;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[path = "compat_http_phase_six/cleanup_receipts.rs"]
mod cleanup_receipts;
#[path = "compat_http_phase_six/ingress_bounds.rs"]
mod ingress_bounds;
#[path = "compat_http_phase_six/integrity.rs"]
mod integrity;
#[path = "compat_http_phase_six/upload_fixtures.rs"]
mod upload_fixtures;

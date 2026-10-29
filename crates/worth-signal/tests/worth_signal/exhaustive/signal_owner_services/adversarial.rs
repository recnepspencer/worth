// Phase 6 adversarial evidence for the public Signal owner-service contract.
//
// This module is intentionally subordinate to `signal_owner_services.rs`.
// Every schedule reaches the public facade and the real sealed owner; the
// operation-control feature only makes an existing owner boundary
// deterministic for the test.
#[allow(unused_imports)]
pub(crate) use crate::fast::signal_owner_services::adversarial::*;

#[path = "adversarial/operation_control.rs"]
pub(crate) mod operation_control;

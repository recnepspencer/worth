#[path = "signal_owner_services/facade_smoke.rs"]
// Public owner-service certification root.
//
// Phase 6 appends only subordinate modules to this intentional integration
// target; production compilation always begins at the curated facade.
pub(crate) mod facade_smoke;

#[path = "signal_owner_services/legacy_cutover.rs"]
pub(crate) mod legacy_cutover;

#[path = "signal_owner_services/legacy_surface.rs"]
pub(crate) mod legacy_surface;

#[path = "signal_owner_services/batch_parity.rs"]
pub(crate) mod batch_parity;

#[path = "signal_owner_services/signal_world/mod.rs"]
pub(crate) mod signal_world;

#[allow(unused_imports)]
pub(crate) use crate::exhaustive::signal_owner_services::independent_oracle;

#[path = "signal_owner_services/adversarial.rs"]
pub(crate) mod adversarial;

#[allow(unused_imports)]
pub(crate) use crate::compile_fail::signal_owner_services::compiler;

#[path = "signal_owner_services/conditional_nested_reuse.rs"]
pub(crate) mod conditional_nested_reuse;

#[path = "../fixtures/signal_owner_services/execution_custody.rs"]
pub(crate) mod execution_custody;

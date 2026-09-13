//! Public owner-service certification root.
//!
//! Phase 6 appends only subordinate modules to this intentional integration
//! target; production compilation always begins at the curated facade.

#[path = "signal_owner_services/facade_smoke.rs"]
mod facade_smoke;

#[path = "signal_owner_services/legacy_cutover.rs"]
mod legacy_cutover;

#[path = "signal_owner_services/legacy_surface.rs"]
mod legacy_surface;

#[path = "signal_owner_services/batch_parity.rs"]
mod batch_parity;

#[path = "signal_owner_services/signal_world/mod.rs"]
mod signal_world;

#[path = "signal_owner_services/independent_oracle.rs"]
mod independent_oracle;

#[path = "signal_owner_services/adversarial.rs"]
mod adversarial;

#[path = "signal_owner_services/compiler.rs"]
mod compiler;

#[path = "signal_owner_services/conditional_nested_reuse.rs"]
mod conditional_nested_reuse;

//! Host-owned trusted-time control for recovery-expiry scenarios (R8.7).
//!
//! Lives in the `ordinary_mutations` tree rather than the shared `support`
//! module because only this test binary exercises authorization-time control.
//! A shared module is compiled into every integration-test binary, so items
//! used by one binary read as dead code in the other ten.

pub use std::sync::{Arc, Mutex};
use worth_query_host::facade::runtime::time::ControlledTime;

pub use bank_server::{BankIdentityRuntime, BankWorldSeed};
pub use worth_query_host::facade::primary_graph::WorthQueryRuntimeClock;

pub use crate::facade::support::{test_world, TestIdentityWorld};

pub type AuthorizationTimeController = std::sync::Arc<ControlledTime>;

/// Install with a host-owned trusted-time source (R8.7 / recovery expiry).
pub fn runtime_with_authorization_time(
    seed: BankWorldSeed,
    source: impl Into<WorthQueryRuntimeClock>,
) -> TestIdentityWorld {
    let runtime = BankIdentityRuntime::install_world_with_authorization_time_source(seed, source)
        .expect("bank test runtime should install with authorization time");
    test_world(runtime)
}

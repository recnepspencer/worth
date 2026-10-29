//! Host-owned trusted-time control for recovery-expiry scenarios (R8.7).
//!
//! Lives in the `ordinary_mutations` tree rather than the shared `support`
//! module because only this test binary exercises authorization-time control.
//! A shared module is compiled into every integration-test binary, so items
//! used by one binary read as dead code in the other ten.

pub use std::sync::{Arc, Mutex};
pub use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use bank_server::{BankIdentityRuntime, BankWorldSeed};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial,
};

pub use crate::facade::server::support::{
    authentication_configuration, CausalBankAuthenticationAdapter, TestIdentityWorld,
};

#[derive(Clone)]
pub struct AuthorizationTimeController {
    pub current: Arc<Mutex<Option<SystemTime>>>,
}

impl AuthorizationTimeController {
    pub fn at_epoch_seconds(seconds: u64) -> Self {
        Self {
            current: Arc::new(Mutex::new(Some(UNIX_EPOCH + Duration::from_secs(seconds)))),
        }
    }

    pub fn advance_to_epoch_seconds(&self, seconds: u64) {
        *self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(UNIX_EPOCH + Duration::from_secs(seconds));
    }

    pub fn make_unavailable(&self) {
        *self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }
}

impl WorthQueryRuntimeTimeSource for AuthorizationTimeController {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        (*self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()))
        .ok_or(WorthQueryRuntimeTimeSourceDenial::Unavailable)
    }
}

/// Install with a host-owned trusted-time source (R8.7 / recovery expiry).
pub fn runtime_with_authorization_time(
    seed: BankWorldSeed,
    source: impl WorthQueryRuntimeTimeSource,
) -> TestIdentityWorld {
    let runtime = BankIdentityRuntime::install_world_with_authorization_time_source(seed, source)
        .expect("bank test runtime should install with authorization time");
    let authentication = runtime
        .admit_authentication_adapter(
            authentication_configuration(),
            CausalBankAuthenticationAdapter,
        )
        .expect("causal authentication adapter should admit");
    TestIdentityWorld {
        runtime,
        authentication,
    }
}

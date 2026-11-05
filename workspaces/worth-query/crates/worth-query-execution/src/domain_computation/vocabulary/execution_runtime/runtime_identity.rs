//! Moved whole from `domain_computation::execution_runtime::runtime_identity`: the items below and every impl they own.
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub(in crate::domain_computation) static NEXT_RUNTIME_AUTHORITY_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// Process-local identity for one concrete Query execution authority owner.
///
/// The identity is minted only while constructing a real execution runtime.
/// Labels, digests, and workspace names cannot reconstruct it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryRuntimeAuthorityIdentity(u64);

impl WorthQueryRuntimeAuthorityIdentity {
    pub(in crate::domain_computation) fn mint() -> Self {
        let ordinal = next_runtime_authority_ordinal(&NEXT_RUNTIME_AUTHORITY_IDENTITY)
            .expect("runtime authority identity space must not be exhausted");
        Self(ordinal)
    }

    /// Test-only mint for axis probes and registry fixtures. Production minting
    /// remains installer-owned via [`super::WorthQueryExecutionRuntimeInstaller`].
    pub fn mint_for_test() -> Self {
        Self::mint()
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

pub(in crate::domain_computation) fn next_runtime_authority_ordinal(counter: &AtomicU64) -> Option<u64> {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = current.checked_add(1) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .ok()
}

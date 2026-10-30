use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_RUNTIME_AUTHORITY_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// Process-local identity for one concrete Query execution authority owner.
///
/// The identity is minted only while constructing a real execution runtime.
/// Labels, digests, and workspace names cannot reconstruct it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryRuntimeAuthorityIdentity(u64);

impl WorthQueryRuntimeAuthorityIdentity {
    pub(super) fn mint() -> Self {
        let ordinal = next_runtime_authority_ordinal(&NEXT_RUNTIME_AUTHORITY_IDENTITY)
            .expect("runtime authority identity space must not be exhausted");
        Self(ordinal)
    }

    /// Test-only mint for axis probes and registry fixtures. Production minting
    /// remains installer-owned via [`super::WorthQueryExecutionRuntimeInstaller`].
    #[cfg(test)]
    pub(crate) fn mint_for_test() -> Self {
        Self::mint()
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

fn next_runtime_authority_ordinal(counter: &AtomicU64) -> Option<u64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_authority_identity_exhaustion_cannot_wrap_or_reuse() {
        let counter = AtomicU64::new(u64::MAX - 1);

        assert_eq!(next_runtime_authority_ordinal(&counter), Some(u64::MAX - 1));
        assert_eq!(next_runtime_authority_ordinal(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}

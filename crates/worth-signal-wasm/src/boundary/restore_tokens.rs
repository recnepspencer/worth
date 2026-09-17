use std::cell::RefCell;
use std::collections::BTreeMap;

use worth_signal::facade::history::RuntimeSnapshot;

use crate::boundary::errors::WorthSignalJsError;
use crate::runtime::core::ExactRuntimeRestoreArtifact;
use crate::runtime::summaries::RuntimeSnapshotEnvelope;

/// How many pending exact restore artifacts one realm keeps. Minting one
/// more evicts the oldest pending artifact; its token then redeems as
/// `restoreTokenNotPending`. Product facades mint a token on every
/// `history.snapshot()`, `branch_snapshot()`, `branch_snapshot_envelope()`
/// and `adapters.exportRuntimeEnvelope()` call, and worker-first roots mint
/// after every mutation, so a hard cap would turn the 65th export of a
/// long-lived realm into a failure unrelated to the caller's own artifacts.
pub const MAXIMUM_PENDING_RESTORE_TOKENS: usize = 64;

thread_local! {
    static RESTORE_TOKENS: RefCell<RestoreTokenRegistry> = RefCell::new(RestoreTokenRegistry::default());
}

struct RestoreTokenRegistry {
    next_token: u64,
    maximum_pending_tokens: usize,
    /// Keyed by token number, so the first entry is always the oldest
    /// pending artifact and eviction is `pop_first`.
    artifacts: BTreeMap<u64, PendingRestoreArtifact>,
}

struct PendingRestoreArtifact {
    prefix: &'static str,
    artifact: RestoreArtifact,
}

enum RestoreArtifact {
    RuntimeEnvelope(ExactRuntimeRestoreArtifact),
    SnapshotEnvelope(RuntimeSnapshotEnvelope),
    Snapshot(RuntimeSnapshot),
    #[cfg(test)]
    Marker,
}

impl Default for RestoreTokenRegistry {
    fn default() -> Self {
        Self {
            next_token: 0,
            maximum_pending_tokens: MAXIMUM_PENDING_RESTORE_TOKENS,
            artifacts: BTreeMap::new(),
        }
    }
}

impl RestoreTokenRegistry {
    /// Stores `artifact` and returns its token. Complexity: O(log pending),
    /// with at most one eviction per store, so the registry never holds more
    /// than `maximum_pending_tokens` artifacts.
    fn store(
        &mut self,
        prefix: &'static str,
        artifact: RestoreArtifact,
    ) -> Result<String, WorthSignalJsError> {
        while self.artifacts.len() >= self.maximum_pending_tokens {
            self.artifacts.pop_first();
        }
        self.next_token = self.next_token.checked_add(1).ok_or_else(|| {
            WorthSignalJsError::internal("restore token identity space exhausted")
        })?;
        self.artifacts
            .insert(self.next_token, PendingRestoreArtifact { prefix, artifact });
        Ok(format!("{prefix}:{}", self.next_token))
    }

    /// Redeems `token`, removing its artifact. A token this realm never
    /// issued for `expected_prefix` is `invalidInput`; one it issued that is
    /// no longer pending (consumed, discarded, or evicted) is
    /// `restoreTokenNotPending`, so a caller can tell a stale artifact from
    /// a foreign one.
    fn take(
        &mut self,
        token: &str,
        expected_prefix: &'static str,
    ) -> Result<RestoreArtifact, WorthSignalJsError> {
        let number = self
            .issued_token_number(token, expected_prefix)
            .ok_or_else(|| unknown_token(expected_prefix, token))?;
        match self.artifacts.get(&number) {
            Some(pending) if pending.prefix == expected_prefix => Ok(self
                .artifacts
                .remove(&number)
                .expect("pending artifact present under the number just observed")
                .artifact),
            Some(_) => Err(unknown_token(expected_prefix, token)),
            None => Err(WorthSignalJsError::restore_token_not_pending(
                token,
                self.maximum_pending_tokens,
            )),
        }
    }

    /// Drops the pending artifact behind `token` if there is one.
    fn discard(&mut self, token: &str) -> bool {
        let Some((prefix, number)) = token.rsplit_once(':') else {
            return false;
        };
        let Ok(number) = number.parse::<u64>() else {
            return false;
        };
        match self.artifacts.get(&number) {
            Some(pending) if pending.prefix == prefix => {
                self.artifacts.remove(&number);
                true
            }
            _ => false,
        }
    }

    fn pending_count(&self) -> usize {
        self.artifacts.len()
    }

    /// The token number when `token` has the shape `{expected_prefix}:{n}`
    /// and `n` is a number this registry has issued.
    fn issued_token_number(&self, token: &str, expected_prefix: &str) -> Option<u64> {
        let number = token
            .strip_prefix(expected_prefix)?
            .strip_prefix(':')?
            .parse::<u64>()
            .ok()?;
        (number >= 1 && number <= self.next_token).then_some(number)
    }
}

pub fn store_runtime_envelope(
    value: ExactRuntimeRestoreArtifact,
) -> Result<String, WorthSignalJsError> {
    RESTORE_TOKENS.with(|registry| {
        registry
            .borrow_mut()
            .store("runtimeEnvelope", RestoreArtifact::RuntimeEnvelope(value))
    })
}

pub fn load_runtime_envelope(
    token: &str,
) -> Result<ExactRuntimeRestoreArtifact, WorthSignalJsError> {
    RESTORE_TOKENS.with(
        |registry| match registry.borrow_mut().take(token, "runtimeEnvelope")? {
            RestoreArtifact::RuntimeEnvelope(artifact) => Ok(artifact),
            _ => unreachable!("runtime envelope token prefix must identify its artifact kind"),
        },
    )
}

pub fn store_snapshot_envelope(
    value: RuntimeSnapshotEnvelope,
) -> Result<String, WorthSignalJsError> {
    RESTORE_TOKENS.with(|registry| {
        registry
            .borrow_mut()
            .store("snapshotEnvelope", RestoreArtifact::SnapshotEnvelope(value))
    })
}

pub fn load_snapshot_envelope(token: &str) -> Result<RuntimeSnapshotEnvelope, WorthSignalJsError> {
    RESTORE_TOKENS.with(
        |registry| match registry.borrow_mut().take(token, "snapshotEnvelope")? {
            RestoreArtifact::SnapshotEnvelope(artifact) => Ok(artifact),
            _ => unreachable!("snapshot envelope token prefix must identify its artifact kind"),
        },
    )
}

pub fn store_snapshot(value: RuntimeSnapshot) -> Result<String, WorthSignalJsError> {
    RESTORE_TOKENS.with(|registry| {
        registry
            .borrow_mut()
            .store("snapshot", RestoreArtifact::Snapshot(value))
    })
}

pub fn load_snapshot(token: &str) -> Result<RuntimeSnapshot, WorthSignalJsError> {
    RESTORE_TOKENS.with(
        |registry| match registry.borrow_mut().take(token, "snapshot")? {
            RestoreArtifact::Snapshot(artifact) => Ok(artifact),
            _ => unreachable!("snapshot token prefix must identify its artifact kind"),
        },
    )
}

/// Releases one pending exact restore artifact. Returns `false` when the
/// token is not pending in this realm (unknown, consumed, discarded, or
/// evicted).
#[wasm_bindgen::prelude::wasm_bindgen(js_name = discardRestoreToken)]
pub fn discard_restore_token(token: String) -> bool {
    RESTORE_TOKENS.with(|registry| registry.borrow_mut().discard(&token))
}

/// How many exact restore artifacts this realm currently holds.
#[wasm_bindgen::prelude::wasm_bindgen(js_name = pendingRestoreTokenCount)]
pub fn pending_restore_token_count() -> usize {
    RESTORE_TOKENS.with(|registry| registry.borrow().pending_count())
}

fn unknown_token(expected_prefix: &str, token: &str) -> WorthSignalJsError {
    WorthSignalJsError::invalid_input(format!("unknown {expected_prefix} restore token `{token}`"))
}

#[cfg(test)]
mod tests {
    use super::{RestoreArtifact, RestoreTokenRegistry};

    fn registry(maximum_pending_tokens: usize) -> RestoreTokenRegistry {
        RestoreTokenRegistry {
            next_token: 0,
            maximum_pending_tokens,
            artifacts: Default::default(),
        }
    }

    #[test]
    fn pending_restore_tokens_are_consumable_and_discardable() {
        let mut registry = registry(2);
        let first = registry.store("test", RestoreArtifact::Marker).unwrap();
        let second = registry.store("test", RestoreArtifact::Marker).unwrap();
        assert_eq!(first, "test:1");
        assert_eq!(second, "test:2");

        // A token redeems under its own prefix only.
        let foreign = registry.take(&first, "other").err().expect("denied");
        assert_eq!(foreign.code, "invalidInput");
        assert!(matches!(
            registry.take(&first, "test").unwrap(),
            RestoreArtifact::Marker
        ));
        // Consumed once: a second redemption is stale, not unknown.
        let stale = registry.take(&first, "test").err().expect("denied");
        assert_eq!(stale.code, "restoreTokenNotPending");

        assert!(registry.discard(&second));
        assert!(!registry.discard(&second));
        assert_eq!(
            registry.take(&second, "test").err().expect("denied").code,
            "restoreTokenNotPending"
        );
        assert_eq!(registry.pending_count(), 0);
    }

    #[test]
    fn minting_past_the_bound_evicts_the_oldest_pending_artifact() {
        let mut registry = registry(2);
        let first = registry.store("test", RestoreArtifact::Marker).unwrap();
        let second = registry.store("test", RestoreArtifact::Marker).unwrap();
        let third = registry.store("test", RestoreArtifact::Marker).unwrap();
        assert_eq!(registry.pending_count(), 2);

        let evicted = registry.take(&first, "test").err().expect("denied");
        assert_eq!(evicted.code, "restoreTokenNotPending");
        assert!(
            evicted.message.contains("2 most recent"),
            "{}",
            evicted.message
        );
        assert!(matches!(
            registry.take(&second, "test").unwrap(),
            RestoreArtifact::Marker
        ));
        assert!(matches!(
            registry.take(&third, "test").unwrap(),
            RestoreArtifact::Marker
        ));
    }

    #[test]
    fn tokens_this_realm_never_issued_are_unknown_not_stale() {
        let mut registry = registry(2);
        registry.store("test", RestoreArtifact::Marker).unwrap();
        for token in ["test:0", "test:2", "test:nope", "test", "other:1", ":1"] {
            let denial = registry.take(token, "test").err().expect("denied");
            assert_eq!(denial.code, "invalidInput", "{token}");
            assert!(!registry.discard(token), "{token}");
        }
        // Prefix mismatch on an issued number is unknown too: the number was
        // never issued under the requested prefix.
        assert_eq!(
            registry
                .take("other:1", "other")
                .err()
                .expect("denied")
                .code,
            "invalidInput"
        );
        assert_eq!(registry.pending_count(), 1);
    }
}

//! The no-retry-claim fact for one terminal head retired.

use worth_store_physical_format::{ReleaseCustodyHeadKeyV1, RootPublicationCell};

use super::super::registry::PhysicalMutationIdempotencyBindingState;
use super::PhysicalMutationIdempotencyRuntimeAuthority;
use crate::physical_runtime::durability::TerminalHeadPublicationExcluded;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum TerminalHeadRetryClaimDenial {
    OwnerReleased,
    UnresolvedBinding,
}

/// Sealed: issued only by the idempotency runtime owner, under its registry
/// lock and behind the installed retirement fence. Every live binding has a
/// settled terminal fate, so no retry can still publish a first release or a
/// later drop for the fenced root. A settled fate does not block: a retry of
/// it answers from the registry and mints nothing.
pub(in crate::physical_runtime) struct TerminalHeadNoRetryClaim {
    key: ReleaseCustodyHeadKeyV1,
    root: RootPublicationCell,
}

impl TerminalHeadNoRetryClaim {
    pub(in crate::physical_runtime) const fn key(&self) -> ReleaseCustodyHeadKeyV1 {
        self.key
    }

    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.root
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) const fn fixture(
        key: ReleaseCustodyHeadKeyV1,
        root: RootPublicationCell,
    ) -> Self {
        Self { key, root }
    }
}

impl PhysicalMutationIdempotencyRuntimeAuthority {
    pub(in crate::physical_runtime) fn attest_no_terminal_head_retry_claim(
        &self,
        publication: &TerminalHeadPublicationExcluded,
    ) -> Result<TerminalHeadNoRetryClaim, TerminalHeadRetryClaimDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(TerminalHeadRetryClaimDenial::OwnerReleased)?;
        let registry = owner
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if registry.bindings.values().any(|state| {
            !matches!(
                state,
                PhysicalMutationIdempotencyBindingState::Terminal { .. }
            )
        }) {
            return Err(TerminalHeadRetryClaimDenial::UnresolvedBinding);
        }
        Ok(TerminalHeadNoRetryClaim {
            key: publication.key(),
            root: publication.root(),
        })
    }
}

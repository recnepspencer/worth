//! The no-hold fact for one terminal head retired.

use worth_store_physical_format::{ReleaseCustodyHeadKeyV1, RootPublicationCell};

use super::{PhysicalProtectedRootObservation, RootProtectionRegistry};
use crate::physical_runtime::durability::{
    CheckpointAttestedTerminalHead, PublicationStateLockHeld,
};

/// Sealed: issued only by the registrations owner. No reader other than the
/// inspecting claim protects a root at or below the attested one, and the
/// release ledger attested that no selected WAL transition above the last
/// checkpoint still needs the head for recovery.
pub(in crate::physical_runtime) struct TerminalHeadNoReaderOrRecoveryHold {
    key: ReleaseCustodyHeadKeyV1,
    root: RootPublicationCell,
}

impl TerminalHeadNoReaderOrRecoveryHold {
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

impl RootProtectionRegistry {
    /// The witness proves the caller holds the publication-state lock, so no
    /// new capture can race this check with installation of the retirement
    /// fence.
    pub(in crate::physical_runtime) fn attest_no_terminal_head_hold(
        &self,
        _held: &PublicationStateLockHeld<'_>,
        inspector: PhysicalProtectedRootObservation,
        attested: &CheckpointAttestedTerminalHead,
    ) -> Option<TerminalHeadNoReaderOrRecoveryHold> {
        if attested.root() != inspector.root()
            || self.external_root_at_or_below(inspector, attested.root().generation().get())
        {
            return None;
        }
        Some(TerminalHeadNoReaderOrRecoveryHold {
            key: attested.key(),
            root: attested.root(),
        })
    }
}

#[cfg(test)]
#[path = "terminal_head_hold/tests.rs"]
mod tests;

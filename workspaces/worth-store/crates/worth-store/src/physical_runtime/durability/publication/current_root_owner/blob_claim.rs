use std::sync::{Arc, Mutex};

use worth_store_physical_format::DurablePhysicalRootManifest;

use super::PhysicalCurrentRootOwner;
use crate::physical_runtime::{
    stability::PhysicalRootReadLease, BlobSessionId, PhysicalReadProtectionDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalBlobSessionClaimDenial {
    CompetingSession,
    Capacity,
    MetadataUnavailable,
    Protection(PhysicalReadProtectionDenial),
    CheckpointChanged,
    ClaimLost,
    ReclaimFenced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalBlobTerminalAdmissionDenial {
    Claim(PhysicalBlobSessionClaimDenial),
    PendingPublication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaimPhase {
    Inspecting,
    Live,
    TerminalPending,
    ReclaimPending,
}

struct ClaimEntry {
    session: [u8; 16],
    phase: ClaimPhase,
}

/// Ephemeral same-session arbitration, bounded by the read-protection profile.
/// Selected C5 records, not these entries, decide durable session fate.
pub(super) struct BlobClaimRegistry {
    capacity: usize,
    entries: Mutex<Vec<ClaimEntry>>,
}

impl BlobClaimRegistry {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Mutex::new(Vec::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<ClaimEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn check_available(&self, session: [u8; 16]) -> Result<(), PhysicalBlobSessionClaimDenial> {
        let entries = self.lock();
        if entries.iter().any(|entry| entry.session == session) {
            return Err(PhysicalBlobSessionClaimDenial::CompetingSession);
        }
        if entries.len() >= self.capacity {
            return Err(PhysicalBlobSessionClaimDenial::Capacity);
        }
        Ok(())
    }

    fn reserve(&self, session: [u8; 16]) -> Result<(), PhysicalBlobSessionClaimDenial> {
        let mut entries = self.lock();
        if entries.iter().any(|entry| entry.session == session) {
            return Err(PhysicalBlobSessionClaimDenial::CompetingSession);
        }
        if entries.len() >= self.capacity {
            return Err(PhysicalBlobSessionClaimDenial::Capacity);
        }
        entries
            .try_reserve(1)
            .map_err(|_| PhysicalBlobSessionClaimDenial::MetadataUnavailable)?;
        entries.push(ClaimEntry {
            session,
            phase: ClaimPhase::Inspecting,
        });
        Ok(())
    }
}

/// One Store-runtime claim. Dropping it only ends live arbitration; it cannot
/// establish an Abort or erase a pending physical-effect obligation.
pub(in crate::physical_runtime) struct PhysicalBlobSessionClaim {
    registry: Arc<BlobClaimRegistry>,
    session: [u8; 16],
}

impl PhysicalBlobSessionClaim {
    pub(in crate::physical_runtime) fn promote_live(
        &mut self,
    ) -> Result<(), PhysicalBlobSessionClaimDenial> {
        let mut entries = self.registry.lock();
        let entry = entries
            .iter_mut()
            .find(|entry| entry.session == self.session)
            .ok_or(PhysicalBlobSessionClaimDenial::ClaimLost)?;
        if entry.phase != ClaimPhase::Inspecting {
            return Err(PhysicalBlobSessionClaimDenial::ClaimLost);
        }
        entry.phase = ClaimPhase::Live;
        Ok(())
    }

    /// Seals this inspection against a later resume while its durable
    /// terminal publication is being prepared and settled. This phase is
    /// process-local arbitration, never evidence of abandonment by itself.
    fn promote_terminal(&mut self) -> Result<(), PhysicalBlobSessionClaimDenial> {
        let mut entries = self.registry.lock();
        let entry = entries
            .iter_mut()
            .find(|entry| entry.session == self.session)
            .ok_or(PhysicalBlobSessionClaimDenial::ClaimLost)?;
        if entry.phase != ClaimPhase::Inspecting {
            return Err(PhysicalBlobSessionClaimDenial::ClaimLost);
        }
        entry.phase = ClaimPhase::TerminalPending;
        Ok(())
    }

    pub(super) fn promote_reclaim(
        &mut self,
        registry: &Arc<BlobClaimRegistry>,
    ) -> Result<(), PhysicalBlobSessionClaimDenial> {
        if !Arc::ptr_eq(&self.registry, registry) {
            return Err(PhysicalBlobSessionClaimDenial::ClaimLost);
        }
        let mut entries = self.registry.lock();
        let entry = entries
            .iter_mut()
            .find(|entry| entry.session == self.session)
            .ok_or(PhysicalBlobSessionClaimDenial::ClaimLost)?;
        if entry.phase != ClaimPhase::Inspecting {
            return Err(PhysicalBlobSessionClaimDenial::ClaimLost);
        }
        entry.phase = ClaimPhase::ReclaimPending;
        Ok(())
    }
}

impl Drop for PhysicalBlobSessionClaim {
    fn drop(&mut self) {
        let mut entries = self.registry.lock();
        if let Some(index) = entries
            .iter()
            .position(|entry| entry.session == self.session)
        {
            entries.swap_remove(index);
        }
    }
}

impl PhysicalCurrentRootOwner {
    /// Registers an inspecting claim and protects exactly the captured root
    /// while holding the current-root lock. A later root publication may
    /// proceed, but cannot invalidate this acquisition's selected scan.
    pub(in crate::physical_runtime) fn capture_blob_inspection(
        &self,
        session: BlobSessionId,
    ) -> Result<
        (
            DurablePhysicalRootManifest,
            PhysicalRootReadLease,
            PhysicalBlobSessionClaim,
        ),
        PhysicalBlobSessionClaimDenial,
    > {
        let state = self.lock_publication_state();
        if self.lock_reclaim().is_some() {
            return Err(PhysicalBlobSessionClaimDenial::ReclaimFenced);
        }
        self.blob_claims.check_available(session.bytes())?;
        let root = state.current_root.clone();
        let protection = self
            .read_protection
            .capture(&root)
            .map_err(PhysicalBlobSessionClaimDenial::Protection)?;
        self.blob_claims.reserve(session.bytes())?;
        drop(state);
        Ok((
            root,
            protection,
            PhysicalBlobSessionClaim {
                registry: Arc::clone(&self.blob_claims),
                session: session.bytes(),
            },
        ))
    }

    /// The live claim already excludes another producer for this session.
    /// A previous uncertain effect must be reconciled before terminal
    /// preparation; unrelated later C.5 work remains independently admitted.
    pub(in crate::physical_runtime) fn promote_blob_terminal(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
    ) -> Result<(), PhysicalBlobTerminalAdmissionDenial> {
        let _state = self.lock_publication_state();
        if !Arc::ptr_eq(&claim.registry, &self.blob_claims) {
            return Err(PhysicalBlobTerminalAdmissionDenial::Claim(
                PhysicalBlobSessionClaimDenial::ClaimLost,
            ));
        }
        if self.publication.pending_len() != 0 {
            return Err(PhysicalBlobTerminalAdmissionDenial::PendingPublication);
        }
        claim
            .promote_terminal()
            .map_err(PhysicalBlobTerminalAdmissionDenial::Claim)
    }
}

#[cfg(test)]
impl PhysicalBlobSessionClaim {
    /// The one claim of `session` in a registry of its own.
    pub(in crate::physical_runtime) fn fixture(session: [u8; 16]) -> Self {
        Self::fixture_in(&Arc::new(BlobClaimRegistry::new(1)), session)
    }

    /// An inspecting claim of `session` in `registry`.
    pub(super) fn fixture_in(registry: &Arc<BlobClaimRegistry>, session: [u8; 16]) -> Self {
        registry.reserve(session).expect("fixture claim");
        Self {
            registry: Arc::clone(registry),
            session,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_table_bounds_competitors_and_drop_releases_slot() {
        let registry = Arc::new(BlobClaimRegistry::new(1));
        let first = [1; 16];
        let second = [2; 16];
        registry.reserve(first).unwrap();
        assert_eq!(
            registry.reserve(first),
            Err(PhysicalBlobSessionClaimDenial::CompetingSession)
        );
        assert_eq!(
            registry.reserve(second),
            Err(PhysicalBlobSessionClaimDenial::Capacity)
        );
        let mut claim = PhysicalBlobSessionClaim {
            registry: Arc::clone(&registry),
            session: first,
        };
        claim.promote_live().unwrap();
        assert_eq!(registry.lock()[0].phase, ClaimPhase::Live);
        drop(claim);
        assert!(registry.lock().is_empty());
        registry.reserve(second).unwrap();
    }

    #[test]
    fn terminal_pending_excludes_competing_session_until_guard_drop() {
        let registry = Arc::new(BlobClaimRegistry::new(2));
        let session = [3; 16];
        registry.reserve(session).unwrap();
        let mut claim = PhysicalBlobSessionClaim {
            registry: Arc::clone(&registry),
            session,
        };
        claim.promote_terminal().unwrap();
        assert_eq!(registry.lock()[0].phase, ClaimPhase::TerminalPending);
        assert_eq!(
            registry.reserve(session),
            Err(PhysicalBlobSessionClaimDenial::CompetingSession)
        );
        drop(claim);
        registry.reserve(session).unwrap();
    }
}

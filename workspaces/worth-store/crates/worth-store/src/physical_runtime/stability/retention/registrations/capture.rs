//! One original registry installation, shared by blocking and try capture.
use super::*;
use crate::physical_runtime::PhysicalCheckpointReadProtectionDenial as Failure;

impl RootProtectionRegistry {
    /// The authoritative current-root mutex must already be held.
    pub(in crate::physical_runtime) fn capture(
        self: &Arc<Self>,
        root: &DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootReadLease, PhysicalReadProtectionDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.install_capture(root, &mut state)
    }
    /// All refusal and liveness checks precede installation. Nothing fallible
    /// runs after creating the accepted lease, while caller gates may be held.
    pub(in crate::physical_runtime) fn try_capture(
        self: &Arc<Self>,
        root: &DurablePhysicalRootManifest,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PhysicalRootReadLease, Failure> {
        request.check_live().map_err(Failure::Request)?;
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Failure::Busy),
        };
        request.check_live().map_err(Failure::Request)?;
        self.install_capture(root, &mut state)
            .map_err(Failure::Protection)
    }
    fn install_capture(
        self: &Arc<Self>,
        root: &DurablePhysicalRootManifest,
        state: &mut Registrations,
    ) -> Result<PhysicalRootReadLease, PhysicalReadProtectionDenial> {
        let lifecycle = self.lifecycle.snapshot();
        if state.revoked || lifecycle.phase != ObservedLifecyclePhase::RecordServing {
            return Err(PhysicalReadProtectionDenial::Revoked);
        }
        let slot = state
            .free
            .ok_or(PhysicalReadProtectionDenial::ProtectionLimit)?;
        let key = root.root_cell();
        state.index_probes = state.index_probes.saturating_add(1);
        let (visited, present) = state.roots.contains(&root.generation());
        state.examined_entries = state.examined_entries.saturating_add(visited);
        let fresh = !present;
        if fresh && state.roots.len() >= self.policy.roots().get() as usize {
            return Err(PhysicalReadProtectionDenial::RetainedRootLimit);
        }
        state.index_probes = state.index_probes.saturating_add(1);
        let tail = fresh.then(|| inline_tail(root)).flatten();
        {
            let protected = state
                .roots
                .entry(root.generation())
                .or_insert_with(|| ProtectedRoot {
                    manifest: root.clone(),
                    acquisitions: 0,
                });
            assert_eq!(
                &protected.manifest, root,
                "one root cell cannot name competing root contents"
            );
            protected.acquisitions += 1;
        }
        if let Some(tail) = tail {
            *state.inline_tails.entry(tail).or_insert(0) += 1;
        }
        state.free = state.slots[slot].next_free;
        state.slots[slot] = RegistrationSlot {
            root: Some(key),
            owners: 1,
            next_free: None,
        };
        state.live += 1;
        state.acquisitions = state.acquisitions.saturating_add(1);
        Ok(PhysicalRootReadLease::new(
            Arc::clone(self),
            slot,
            PhysicalProtectedRootObservation::new(self.runtime, lifecycle.generation, key),
        ))
    }
}

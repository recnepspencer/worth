//! Move-only recovery handle (R8.28 / R8.29). Neither Clone nor Copy.

use std::sync::Arc;

use worth_proof::LinearResource;
use worth_query_installation::facade::WorthQueryCanonicalWorkPhases;

use crate::domain_computation::managed_run::{
    WorthQueryRecoveryHandleRegistry, WorthQueryRecoveryRegistrySlot,
    WorthQueryRecoveryResourceTerminal,
};

use super::binding::WorthQueryRecoveryHandleBinding;
use super::denial::{WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind};
use super::identity::{
    WorthQueryOpaqueRecoveryWireIdentity, WorthQueryRecoveryHandleAuthorityIdentity,
    WorthQueryRecoveryHandleIdentity,
};

worth_proof::authority_marker!(WorthQueryRecoveryLifecycleAuthority);

type WorthQueryLinearRecoveryResource = LinearResource<
    WorthQueryRecoveryHandleIdentity,
    WorthQueryRecoveryResourceTerminal,
    WorthQueryRecoveryLifecycleAuthority,
>;

/// Framework-owned linear recovery handle.
///
/// Transitions that produce effect authority or terminate the resource take
/// `self` by value so a second transition is unrepresentable after consume.
pub struct WorthQueryRecoveryHandle {
    lifecycle: Option<WorthQueryLinearRecoveryResource>,
    slot: WorthQueryRecoveryRegistrySlot,
    registry: Arc<WorthQueryRecoveryHandleRegistry>,
    binding: WorthQueryRecoveryHandleBinding,
    canonical_work: WorthQueryCanonicalWorkPhases,
}

impl WorthQueryRecoveryHandle {
    pub(super) fn new(
        identity: WorthQueryRecoveryHandleIdentity,
        slot: WorthQueryRecoveryRegistrySlot,
        registry: Arc<WorthQueryRecoveryHandleRegistry>,
        binding: WorthQueryRecoveryHandleBinding,
        canonical_work: WorthQueryCanonicalWorkPhases,
    ) -> Self {
        Self {
            lifecycle: Some(LinearResource::mint(
                identity,
                &WorthQueryRecoveryLifecycleAuthority::witness(),
            )),
            slot,
            registry,
            binding,
            canonical_work,
        }
    }

    pub fn binding(&self) -> &WorthQueryRecoveryHandleBinding {
        &self.binding
    }

    /// Execution-runtime authority identity this handle's registry belongs to (Q8.20).
    pub(crate) fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.registry.runtime_authority()
    }

    /// Owner-test observation of this handle's instance registry.
    #[cfg(test)]
    pub(crate) fn registry_arc(&self) -> Arc<WorthQueryRecoveryHandleRegistry> {
        Arc::clone(&self.registry)
    }

    /// Owner-test observation of this handle's opaque membership slot.
    #[cfg(test)]
    pub(crate) const fn registry_slot(&self) -> WorthQueryRecoveryRegistrySlot {
        self.slot
    }

    /// The owning runtime's clock, reached through the registry (R8.31).
    ///
    /// This is what lets `ensure_for` re-sample the deadline at *use* without a
    /// transition ever taking a clock or a timestamp from its caller.
    pub(crate) fn registry_clock(
        &self,
    ) -> &crate::domain_computation::runtime_time::WorthQueryRuntimeClock {
        self.registry.clock()
    }

    pub fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }

    /// Opaque wire projection. Cannot be readmitted as a handle (R8.34).
    pub fn opaque_wire_identity(&self) -> WorthQueryOpaqueRecoveryWireIdentity {
        WorthQueryOpaqueRecoveryWireIdentity::project(self.identity())
    }

    pub(crate) fn identity(&self) -> &WorthQueryRecoveryHandleIdentity {
        self.lifecycle
            .as_ref()
            .expect("live handle owns one linear resource")
            .id()
    }

    pub(crate) fn authority_identity(&self) -> WorthQueryRecoveryHandleAuthorityIdentity {
        WorthQueryRecoveryHandleAuthorityIdentity::from_handle(self.identity())
    }

    pub(crate) fn ensure_live(&self) -> Result<(), WorthQueryRecoveryHandleDenial> {
        if self.registry.is_live(self.slot) {
            Ok(())
        } else {
            Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::AlreadyTerminal,
            ))
        }
    }

    /// Leave without consuming the recovery, returning the mint claim.
    ///
    /// Not a transition and not reachable from outside the crate: the only
    /// callers are the [`RelinquishOnDenial`] combinators.
    fn relinquish(mut self) {
        if let Some(resource) = self.lifecycle.take() {
            let _ = resource.terminate(WorthQueryRecoveryResourceTerminal::Relinquished);
            self.registry.relinquish(self.slot);
        }
    }

    pub(crate) fn consume(
        mut self,
        terminal: WorthQueryRecoveryResourceTerminal,
    ) -> Result<WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial> {
        let resource = self
            .lifecycle
            .take()
            .expect("consuming handle owns one linear resource");
        if !self.registry.mark_terminal(self.slot, terminal) {
            let recorded = self
                .registry
                .terminal_of(self.slot)
                .unwrap_or(WorthQueryRecoveryResourceTerminal::ForceTerminated);
            let _ = resource.terminate(recorded);
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::AlreadyTerminal,
            ));
        }
        let receipt = resource.terminate(terminal);
        debug_assert_eq!(receipt.terminal(), &terminal);
        Ok(self.binding.clone())
    }

    /// Named fixture handle for resolve binding proofs. Not production.
    #[cfg(test)]
    pub(crate) fn axis_probe(binding: WorthQueryRecoveryHandleBinding) -> Self {
        let registry = Arc::new(WorthQueryRecoveryHandleRegistry::new());
        Self::axis_probe_in_registry(binding, registry)
    }

    #[cfg(test)]
    pub(crate) fn axis_probe_in_registry(
        binding: WorthQueryRecoveryHandleBinding,
        registry: Arc<WorthQueryRecoveryHandleRegistry>,
    ) -> Self {
        use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

        use super::identity::WorthQueryRecoveryHandleIdentity;

        let slot = registry.register_axis_probe();
        Self::new(
            WorthQueryRecoveryHandleIdentity::mint(),
            slot,
            registry,
            binding,
            WorthQueryCanonicalWorkPhases::new(
                WorthQueryCanonicalWorkEvidence::zero(),
                WorthQueryCanonicalWorkEvidence::zero(),
                WorthQueryCanonicalWorkEvidence::zero(),
                WorthQueryCanonicalWorkEvidence::zero(),
                WorthQueryCanonicalWorkEvidence::zero(),
            ),
        )
    }
}

/// Denial gives the handle back to the commit (Q8.21-L11).
///
/// Every consuming transition denies *before* it consumes. Letting the moved
/// handle simply fall out of scope on those paths ran `Drop`, which recorded
/// `Disposed` and kept the authoritative mint claim — so one attempt with stale
/// or wrong authority destroyed the commit's recovery permanently. Routing
/// denials through here keeps the move (a caller still cannot transition twice
/// with one value) while leaving the commit as recoverable as it was before the
/// attempt.
///
/// A trait rather than three copies of the same combinator: undo and redo hold
/// the handle inside a larger admitted value, and this rule is exactly the kind
/// that must not drift between them. Implementors supply only *where their
/// handle is*; the policy has one owner.
pub(crate) trait RelinquishOnDenial: Sized {
    /// Give up the held handle without consuming the recovery it stands for.
    fn relinquish_held_handle(self);

    /// Run by-reference admission checks, returning `self` on success and
    /// relinquishing on denial.
    ///
    /// Generic over the denial type because undo and redo map handle denials
    /// into their own taxonomies before returning.
    fn admit<Denial>(
        self,
        check: impl FnOnce(&Self) -> Result<(), Denial>,
    ) -> Result<Self, Denial> {
        self.admit_deriving(check).map(|(held, ())| held)
    }

    /// [`Self::admit`] for checks that also derive a value on the way through.
    fn admit_deriving<Derived, Denial>(
        self,
        check: impl FnOnce(&Self) -> Result<Derived, Denial>,
    ) -> Result<(Self, Derived), Denial> {
        match check(&self) {
            Ok(derived) => Ok((self, derived)),
            Err(denial) => {
                self.relinquish_held_handle();
                Err(denial)
            }
        }
    }
}

impl RelinquishOnDenial for WorthQueryRecoveryHandle {
    fn relinquish_held_handle(self) {
        self.relinquish();
    }
}

impl Drop for WorthQueryRecoveryHandle {
    fn drop(&mut self) {
        if let Some(resource) = self.lifecycle.take() {
            let terminal = self
                .registry
                .terminal_of(self.slot)
                .unwrap_or(WorthQueryRecoveryResourceTerminal::Disposed);
            let receipt = resource.terminate(terminal);
            let _ = self.registry.mark_terminal(self.slot, *receipt.terminal());
        }
    }
}

impl std::fmt::Debug for WorthQueryRecoveryHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRecoveryHandle")
            .field("slot", &self.slot)
            .field("registry_live", &self.registry.is_live(self.slot))
            .finish_non_exhaustive()
    }
}

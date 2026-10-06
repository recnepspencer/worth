mod branch_authority;
mod close_authority;
mod configuration;
mod core_access;
mod merge_authority;
mod owner_lifecycle;
mod owner_seal;
mod partition_edition_access;
mod preparation_runtime;
mod publication_lifecycle;
mod publication_settlement_registry;
mod tenure;
#[cfg(test)]
mod test_support;

use super::{
    CommitStrategiesSubsystem, DurabilitySubsystem, HistorySubsystem, IndexingSubsystem,
    LineageSubsystem, PublicationSubsystem, RecordIdentitySubsystem, RuntimeServices,
    StorageSubsystem, VisibilitySubsystem,
};

pub(in crate::runtime) use close_authority::{
    RelationalRuntimeCloseAuthority, RelationalRuntimeSeal,
};
pub(crate) use configuration::{
    RelationalRuntimeConfiguration, RelationalRuntimeConfigurationBinding,
    RelationalRuntimeConfigurationSnapshot,
};
pub(in crate::runtime) use owner_lifecycle::RelationalRuntimeOwner;
pub(crate) use owner_lifecycle::{
    AdmittedRelationalRuntimeOperation, RelationalRuntimeAdmissionPosture,
    RelationalRuntimeOwnerBinding,
};
pub use owner_seal::{RelationalRuntimeSealDenial, RelationalRuntimeSealOutcome};
pub(crate) use preparation_runtime::RelationalPreparationOwnerBinding;
pub(crate) use preparation_runtime::RelationalPreparationRuntime;
pub(in crate::runtime) use publication_lifecycle::RelationalRuntimePublicationOwner;
pub(crate) use publication_lifecycle::{
    RelationalCandidateRegistrationDenial, RelationalRuntimePublicationBinding,
};
pub(crate) use publication_settlement_registry::{
    DeferredRelationalSettlement, PendingRelationalPublicationSettlement,
    PerformedRelationalSettlement, RelationalPendingSettlementReservation,
    RelationalPublicationSettlementRegistry, RelationalSettlementClaim,
    RelationalSettlementReservationDenial, ReservedRelationalSettlement,
};
pub(in crate::runtime) use tenure::RelationalRuntimeTenure;

/// Every subsystem one relational runtime owns.
///
/// The state is held by shared ownership so an independently borrowable owner
/// service can address this exact authority without copying it and without a
/// runtime-wide lock. Only the owning [`RelationalRuntime`] keeps a lasting
/// strong handle; a service binds weakly and answers owner-unavailable once the
/// owner is gone.
#[derive(Debug)]
pub struct RelationalRuntimeState {
    /// The one authority for this runtime's configuration and the schema
    /// contract runtime derived from it.
    pub(crate) configuration: RelationalRuntimeConfiguration,
    pub(crate) commit_strategies: CommitStrategiesSubsystem,
    pub(crate) partitions: StorageSubsystem,
    pub(crate) visibility: VisibilitySubsystem,
    pub(crate) publication: PublicationSubsystem,
    pub(crate) history: HistorySubsystem,
    pub(crate) indexes: IndexingSubsystem,
    pub(crate) lineage: LineageSubsystem,
    pub(crate) durability: DurabilitySubsystem,
    pub(crate) record_identity: RecordIdentitySubsystem,
    pub(crate) services: RuntimeServices,
    pub(in crate::runtime) owner_lifecycle: RelationalRuntimeOwner,
    pub(in crate::runtime) publication_owner: RelationalRuntimePublicationOwner,
}

/// A handle onto one relational runtime, together with the tenure that entitles
/// it to exist.
///
/// The owner's handle carries close authority, so finishing the runtime is an
/// owner lifecycle event on the owner's own thread. A service that is
/// mid-operation receives a handle carrying its admission instead, which keeps
/// the state alive for exactly that operation and closes nothing.
#[derive(Debug)]
pub struct RelationalRuntime {
    /// The configuration in force for this handle, and the schema contract
    /// runtime lowered from it.
    ///
    /// The one authority is [`RelationalRuntimeState::configuration`]; this is
    /// a read of it taken when the handle was made, not a second copy of the
    /// truth. An owner refreshes it in the same call that installs a change, so
    /// it can never answer from a configuration it has already replaced. A
    /// handle admitted for one operation keeps what was in force when it was
    /// admitted, so that operation never sees the registry and the contract
    /// runtime lowered from it disagree.
    pub(crate) config: std::sync::Arc<crate::runtime::RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: std::sync::Arc<super::SchemaContractRuntimeSubsystem>,
    state: std::sync::Arc<RelationalRuntimeState>,
    tenure: RelationalRuntimeTenure,
}

impl RelationalRuntime {
    pub(in crate::runtime) fn from_state(state: RelationalRuntimeState) -> Self {
        let close = RelationalRuntimeCloseAuthority::new(
            state.services.runtime_instance_id(),
            state.owner_lifecycle.binding(),
            state.publication_owner.binding(),
        );
        let installed = state.configuration.snapshot();
        Self {
            config: installed.config,
            schema_contract_runtime: installed.schema_contract_runtime,
            state: std::sync::Arc::new(state),
            tenure: RelationalRuntimeTenure::Owner(close),
        }
    }

    /// A handle for one admitted operation against state the owner still holds.
    ///
    /// The admission is carried by the handle, so the operation cannot outlive
    /// its admission and the handle cannot close the runtime it borrows.
    pub(crate) fn admitted(
        state: std::sync::Arc<RelationalRuntimeState>,
        operation: AdmittedRelationalRuntimeOperation,
    ) -> Self {
        let installed = state.configuration.snapshot();
        Self {
            config: installed.config,
            schema_contract_runtime: installed.schema_contract_runtime,
            state,
            tenure: RelationalRuntimeTenure::Admitted(operation),
        }
    }

    /// Install a configuration change and take up its result.
    ///
    /// The exclusive handle is what entitles the change, and this one call both
    /// installs it in the runtime's single configuration authority and refreshes
    /// what this handle reads. Reconfiguration has no other route, so no owner
    /// can install a change and go on answering from the configuration it
    /// replaced.
    pub(in crate::runtime) fn reconfigure(
        &mut self,
        install: impl FnOnce(&RelationalRuntimeConfiguration),
    ) {
        install(&self.state.configuration);
        let installed = self.state.configuration.snapshot();
        self.config = installed.config;
        self.schema_contract_runtime = installed.schema_contract_runtime;
    }

    /// A weak binding to this runtime's exact state, for a service that must
    /// outlive its owner and deny once the owner is gone.
    pub(crate) fn state_binding(&self) -> std::sync::Weak<RelationalRuntimeState> {
        std::sync::Arc::downgrade(&self.state)
    }
}

impl Drop for RelationalRuntime {
    /// Finish an owner before it releases anything its close depends on.
    ///
    /// A type's own `Drop` runs before any of its fields, so by the time the
    /// state, the configuration snapshot and the tenure are released, admission
    /// has already stopped and every admitted operation has already returned.
    /// The close therefore never depends on which field is declared first, and
    /// never runs against a half-released runtime.
    ///
    /// A sealed owner skips both closes explicitly: its seal already stopped
    /// admission with nothing in flight and resolved publication settlement,
    /// so it neither waits nor resolves settlement a second time.
    ///
    /// An operation handle reaches this too, and does nothing: it carries an
    /// admission rather than close authority, so releasing it is exactly what
    /// lets a waiting owner finish.
    fn drop(&mut self) {
        match &self.tenure {
            RelationalRuntimeTenure::Owner(close) => close.close(),
            RelationalRuntimeTenure::Sealed(seal) => seal.finish(),
            RelationalRuntimeTenure::Admitted(_) => {}
        }
    }
}

impl std::ops::Deref for RelationalRuntime {
    type Target = RelationalRuntimeState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl RelationalRuntime {
    /// Whether this handle is the owner's, and so carries the authority that
    /// closes the runtime when it is dropped.
    pub(crate) const fn is_owner(&self) -> bool {
        self.tenure.close_authority().is_some()
    }

    /// Exclusive access to state no one else holds a handle to.
    ///
    /// Recovery rebuilds a runtime from a plan before any service can bind to
    /// it, so it still holds the only handle and may install whole subsystems
    /// into it. The `#[cfg(test)]` retention-capacity seam uses the same door.
    ///
    /// This does not claim that an exclusive runtime borrow proves exclusive
    /// state access, and the two conditions are separate. An operation handle
    /// is refused outright by the owner check. Beyond that, [`Arc::get_mut`]
    /// answers `Some` only while this is the sole strong handle to the state
    /// and nothing holds a [`Weak`] one, so the answer tracks the handles that
    /// actually exist rather than the borrow.
    ///
    /// Only [`Self::state_binding`] mints such a handle, and only
    /// `RelationalSettlementPort` takes one. A live settlement port therefore
    /// forces `None` here; the preparation, fork and publication ports clone
    /// subsystem handles instead and do not. Live reconfiguration never comes
    /// through here at all, and goes through [`Self::reconfigure`] instead.
    ///
    /// [`Arc::get_mut`]: std::sync::Arc::get_mut
    /// [`Weak`]: std::sync::Weak
    pub(crate) fn unshared_state_mut(&mut self) -> Option<&mut RelationalRuntimeState> {
        if !self.is_owner() {
            return None;
        }
        std::sync::Arc::get_mut(&mut self.state)
    }
}

impl RelationalRuntime {
    pub(crate) fn configuration_binding(&self) -> RelationalRuntimeConfigurationBinding {
        self.configuration.binding()
    }

    pub(crate) fn preparation_runtime_snapshot(&self) -> RelationalPreparationRuntime {
        RelationalPreparationOwnerBinding::from_runtime(self).runtime_snapshot()
    }

    pub(crate) fn owner_binding(&self) -> RelationalRuntimeOwnerBinding {
        self.owner_lifecycle.binding()
    }

    pub(crate) fn publication_binding(&self) -> RelationalRuntimePublicationBinding {
        self.publication_owner.binding()
    }
}

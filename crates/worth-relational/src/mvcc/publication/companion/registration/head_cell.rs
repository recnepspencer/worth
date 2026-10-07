//! One Native-selected head and publication exclusion through caller installation.
use super::super::CompanionBranchCell;
use super::{
    CompanionRegistrationState, PendingCompanionRegistration, PublicationCompanionRegistration,
    PublicationCompanionRegistrationPort, PublicationCompanionRegistrationStop as Stop,
};
use crate::branch::RelationalBranchIdentity;
use crate::runtime::{PositionedRelationalSnapshot, RelationalRuntime};
use std::sync::Arc;

impl PublicationCompanionRegistration {
    /// Mint only at the current live Native head, retaining exclusion through `install`.
    ///
    /// Lock order: the caller's Query runtime mutex (when present), then
    /// configuration epoch, exclusive companion publication epoch,
    /// then the caller's branch lookup. Publishers take a shared companion epoch
    /// before preflight enters that same lookup; cutover takes no caller lookup.
    /// Callers must reserve ledger capacity before entry and hold no lookup lock.
    /// `install` only inserts prepaid lookup state: it must not publish, read
    /// through Relational, or wait for a request. Publication contention is a
    /// nonblocking stop and never runs `install` or retains `initial`.
    pub fn with_branch_cell_at_head<T: Send + Sync + 'static, R>(
        &self,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
        initial: Arc<T>,
        install: impl FnOnce(CompanionBranchCell<T>) -> R,
    ) -> Result<R, Stop> {
        self.port
            .with_head_cell(self.generation, runtime, branch, initial, install)
    }
}

impl PendingCompanionRegistration {
    /// Initial registration also selects Native's current head. A snapshot
    /// observed before required registration began is not head evidence.
    /// Exclusion and callback requirements match the active registration door.
    pub fn with_branch_cell_at_head<T: Send + Sync + 'static, R>(
        &self,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
        initial: Arc<T>,
        install: impl FnOnce(CompanionBranchCell<T>) -> R,
    ) -> Result<R, Stop> {
        self.port
            .with_head_cell(self.generation, runtime, branch, initial, install)
    }
}

impl PublicationCompanionRegistrationPort {
    fn with_head_cell<T: Send + Sync + 'static, R>(
        &self,
        generation: u64,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
        initial: Arc<T>,
        install: impl FnOnce(CompanionBranchCell<T>) -> R,
    ) -> Result<R, Stop> {
        let _operation = self.owner.admit().ok_or(Stop::OwnerUnavailable)?;
        if runtime.runtime_instance_id() != self.runtime_instance_id
            || branch.runtime_instance_id() != self.runtime_instance_id
            || !self
                .publication
                .belongs_to_same_owner(&runtime.publication_binding())
        {
            return Err(Stop::ForeignRuntime);
        }
        let configuration = runtime.configuration_binding();
        let _configuration_epoch = configuration.operation();
        let registry = self.publication.companion_registry();
        // A prepared or cutting-over publisher holds a read epoch. This must
        // never wait behind it, including an independently owned Native port.
        let exclusion = registry
            .state
            .try_write()
            .map_err(|_| Stop::HeadCellPublicationContended)?;
        match &*exclusion {
            CompanionRegistrationState::RequiredRebind { generation: active }
            | CompanionRegistrationState::RequiredActive {
                generation: active, ..
            } if *active == generation => {}
            _ => return Err(Stop::Superseded),
        }
        let basis = runtime
            .admit_branch_basis(branch)
            .map_err(|_| Stop::HeadUnavailable)?;
        let root = &basis.inner.root;
        let observation = basis.observation();
        let commit_id = observation.commit_id();
        let position = match commit_id {
            Some(commit_id) => Some(
                runtime
                    .history
                    .canonical_stream_position(commit_id)
                    .ok_or(Stop::HeadUnavailable)?,
            ),
            None => None,
        };
        let selected = PositionedRelationalSnapshot {
            runtime_instance_id: self.runtime_instance_id,
            branch_id: branch.branch_id().clone(),
            root_id: root.id(),
            version_id: observation.version_id(),
            commit_id,
            position,
        };
        let retention = registry.reserve_cell()?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            install(CompanionBranchCell::new(
                &selected, generation, initial, retention,
            ))
        }));
        // Release normally before resuming the caller's unwind: publication
        // exclusion must not be poisoned by arbitrary derived installation.
        drop(exclusion);
        match result {
            Ok(value) => Ok(value),
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
}

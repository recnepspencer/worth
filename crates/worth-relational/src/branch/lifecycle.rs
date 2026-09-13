use crate::history::data::BranchId;

/// Runtime-local posture of one mutable branch reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBranchLifecyclePosture {
    Live,
    Archived,
    Deleting,
}

impl super::reference::RelationalBranchReferenceMutableState {
    pub(super) fn archive(
        &mut self,
    ) -> Result<super::RelationalBranchReferenceObservation, super::RelationalBranchCellDenial>
    {
        let generation = self
            .observation
            .generation()
            .checked_advance()
            .map_err(|_| super::RelationalBranchCellDenial::GenerationOverflow)?;
        self.observation = super::RelationalBranchReferenceObservation::new(
            self.observation.branch_id().clone(),
            self.observation.target().clone(),
            generation,
        );
        self.lifecycle = RelationalBranchLifecyclePosture::Archived;
        Ok(self.observation.clone())
    }

    pub(super) fn mark_deleting(&mut self) {
        self.lifecycle = RelationalBranchLifecyclePosture::Deleting;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchArchiveDenial {
    OwnerUnavailable,
    ForeignRuntime,
    UnknownBranch(BranchId),
    AlreadyArchived(BranchId),
    Deleting(BranchId),
    GenerationOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchDeleteDenial {
    OwnerUnavailable,
    ForeignRuntime,
    UnknownBranch(BranchId),
    MainBranch,
    RetentionBackpressure,
    RetentionIdentityExhausted,
    RetiredIdentityCapacityExhausted,
    OwnerFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivedRelationalBranch {
    identity: super::RelationalBranchIdentity,
    observation: super::RelationalBranchReferenceObservation,
}

impl ArchivedRelationalBranch {
    pub fn identity(&self) -> &super::RelationalBranchIdentity {
        &self.identity
    }

    pub fn observation(&self) -> &super::RelationalBranchReferenceObservation {
        &self.observation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedRelationalBranch {
    identity: super::RelationalBranchIdentity,
    retired_root_identity: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalBranchDeletionPending {
    identity: super::RelationalBranchIdentity,
    active_operation_count: u64,
}

impl RelationalBranchDeletionPending {
    pub fn identity(&self) -> &super::RelationalBranchIdentity {
        &self.identity
    }

    pub const fn active_operation_count(&self) -> u64 {
        self.active_operation_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchDeletionOutcome {
    Deleted(DeletedRelationalBranch),
    WaitingForActiveOperations(RelationalBranchDeletionPending),
}

impl RelationalBranchDeletionOutcome {
    pub fn deleted(&self) -> Option<&DeletedRelationalBranch> {
        match self {
            Self::Deleted(deleted) => Some(deleted),
            Self::WaitingForActiveOperations(_) => None,
        }
    }

    pub fn waiting(&self) -> Option<&RelationalBranchDeletionPending> {
        match self {
            Self::Deleted(_) => None,
            Self::WaitingForActiveOperations(waiting) => Some(waiting),
        }
    }
}

impl DeletedRelationalBranch {
    pub fn identity(&self) -> &super::RelationalBranchIdentity {
        &self.identity
    }

    pub const fn retired_root_identity(&self) -> u64 {
        self.retired_root_identity
    }
}

impl crate::runtime::RelationalRuntime {
    pub fn archive_branch(
        &mut self,
        identity: &super::RelationalBranchIdentity,
    ) -> Result<ArchivedRelationalBranch, RelationalBranchArchiveDenial> {
        require_local_identity(self, identity)
            .map_err(|_| RelationalBranchArchiveDenial::ForeignRuntime)?;
        let publication_cell = self
            .history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .map(|cell| cell.publication_cell())
            .ok_or_else(|| {
                RelationalBranchArchiveDenial::UnknownBranch(identity.branch_id().clone())
            })?;
        let _coordination = publication_cell.coordination().enter();
        let mut state = publication_cell.enter_state();
        match state.lifecycle_posture() {
            RelationalBranchLifecyclePosture::Live => {}
            RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalBranchArchiveDenial::AlreadyArchived(
                    identity.branch_id().clone(),
                ));
            }
            RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalBranchArchiveDenial::Deleting(
                    identity.branch_id().clone(),
                ));
            }
        }
        let observation = state
            .archive()
            .map_err(|_| RelationalBranchArchiveDenial::GenerationOverflow)?;
        Ok(ArchivedRelationalBranch {
            identity: identity.clone(),
            observation,
        })
    }

    pub fn delete_branch(
        &mut self,
        identity: &super::RelationalBranchIdentity,
    ) -> Result<RelationalBranchDeletionOutcome, RelationalBranchDeleteDenial> {
        require_local_identity(self, identity)
            .map_err(|_| RelationalBranchDeleteDenial::ForeignRuntime)?;
        if identity.branch_id() == &self.config.history.main_branch {
            return Err(RelationalBranchDeleteDenial::MainBranch);
        }
        let publication_cell = self
            .history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .map(|cell| cell.publication_cell())
            .ok_or_else(|| {
                RelationalBranchDeleteDenial::UnknownBranch(identity.branch_id().clone())
            })?;
        let _coordination = publication_cell.coordination().enter();
        if self
            .history
            .branch_cell(identity.branch_id())
            .is_none_or(|cell| cell.identity() != identity)
        {
            return Err(RelationalBranchDeleteDenial::UnknownBranch(
                identity.branch_id().clone(),
            ));
        }
        let active_operation_count = publication_cell
            .head_retention()
            .binding()
            .map_err(|_| RelationalBranchDeleteDenial::OwnerFailure)?
            .active_operation_count(identity)
            .map_err(|_| RelationalBranchDeleteDenial::OwnerFailure)?;
        if active_operation_count > 0 {
            self.history
                .reserve_branch_name_retirement(identity.branch_id().clone())
                .map_err(|()| RelationalBranchDeleteDenial::RetiredIdentityCapacityExhausted)?;
            publication_cell.enter_state().mark_deleting();
            return Ok(RelationalBranchDeletionOutcome::WaitingForActiveOperations(
                RelationalBranchDeletionPending {
                    identity: identity.clone(),
                    active_operation_count,
                },
            ));
        }
        let previous_head_version = self
            .history()
            .branch_head(identity.branch_id())
            .map(|head| head.version_id);
        let selected_root = publication_cell
            .selected_root()
            .ok_or(RelationalBranchDeleteDenial::OwnerFailure)?;
        let head_retirement = self
            .history
            .reserve_branch_head_retirement(
                identity,
                &selected_root,
                publication_cell.head_retention(),
            )
            .map_err(|denial| match denial {
                crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
                    RelationalBranchDeleteDenial::RetentionBackpressure
                }
                crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
                    RelationalBranchDeleteDenial::RetentionIdentityExhausted
                }
                _ => RelationalBranchDeleteDenial::OwnerFailure,
            })?;
        self.history
            .reserve_branch_name_retirement(identity.branch_id().clone())
            .map_err(|()| RelationalBranchDeleteDenial::RetiredIdentityCapacityExhausted)?;
        publication_cell.enter_state().mark_deleting();
        let removed_cell = self
            .history
            .remove_branch_cell(identity.branch_id())
            .expect("coordinated branch cell remains registered until deletion cutover");
        let removed_root = removed_cell
            .root()
            .expect("deleted live branch cell retains its selected root");
        assert!(
            std::sync::Arc::ptr_eq(&selected_root, &removed_root),
            "coordinated deletion retires the root that reserved capacity"
        );
        let retired_root_identity = head_retirement.retire_head(removed_root);
        self.history
            .move_branch_head_version(previous_head_version, None);
        self.visibility_pins()
            .move_branch_head_visibility_residency(
                identity.branch_id(),
                previous_head_version,
                None,
                None,
            );
        Ok(RelationalBranchDeletionOutcome::Deleted(
            DeletedRelationalBranch {
                identity: identity.clone(),
                retired_root_identity,
            },
        ))
    }
}

fn require_local_identity(
    runtime: &crate::runtime::RelationalRuntime,
    identity: &super::RelationalBranchIdentity,
) -> Result<(), ()> {
    (identity.runtime_instance_id() == runtime.runtime_instance_id())
        .then_some(())
        .ok_or(())
}

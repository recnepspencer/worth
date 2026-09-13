use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use crate::history::data::{BranchCreateError, BranchId, CommitId};

use super::authority::admit_relational_fork_source;
use super::fork_source_basis::{AdmittedRelationalForkSourceBasis, RelationalForkSourceDescriptor};
use super::identity::RelationalBranchIdentity;
use super::reference::{RelationalBranchCellDenial, RelationalBranchReferenceObservation};
use super::RelationalBranchVersion;
use super::RelationalForkPort;

#[path = "fork_target_installation.rs"]
mod target_installation;

/// Typed denials for the Phase-4 fork transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalForkDenial {
    SourceBranchMissing,
    SourceArchived,
    SourceDeleting,
    EmptySource,
    DuplicateTarget,
    RetiredTarget,
    ForeignRuntime,
    StaleSource,
    MissingArtifact,
    InvalidTarget(BranchCreateError),
    Cell(RelationalBranchCellDenial),
    RetentionCapacityExhausted,
    RetentionOwnerUnavailable,
    RetentionIdentityExhausted,
    RetentionInvariantViolation,
    OwnerUnavailable,
}

/// Read-only evidence returned after a successful branch-reference fork.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalForkOutcome {
    target_identity: RelationalBranchIdentity,
    source_observation: RelationalBranchReferenceObservation,
    target_observation: RelationalBranchReferenceObservation,
    fork_provenance: RelationalBranchReferenceObservation,
    source_truth_version: RelationalBranchVersion,
    target_truth_version: RelationalBranchVersion,
    shared_commit_id: Option<CommitId>,
}

impl RelationalForkOutcome {
    pub fn target_identity(&self) -> &RelationalBranchIdentity {
        &self.target_identity
    }

    pub fn source_observation(&self) -> &RelationalBranchReferenceObservation {
        &self.source_observation
    }

    pub fn target_observation(&self) -> &RelationalBranchReferenceObservation {
        &self.target_observation
    }

    pub fn fork_provenance(&self) -> &RelationalBranchReferenceObservation {
        &self.fork_provenance
    }

    pub const fn source_truth_version(&self) -> RelationalBranchVersion {
        self.source_truth_version
    }

    pub const fn target_truth_version(&self) -> RelationalBranchVersion {
        self.target_truth_version
    }

    pub const fn shared_commit_id(&self) -> Option<CommitId> {
        self.shared_commit_id
    }
}

impl RelationalForkPort {
    /// Observe an exact live source and issue the owner-sealed fork-only
    /// token. Empty branches do not produce a fork source.
    pub fn observe_fork_source(
        &self,
        source_branch: &BranchId,
    ) -> Result<
        (
            RelationalForkSourceDescriptor,
            AdmittedRelationalForkSourceBasis,
        ),
        RelationalForkDenial,
    > {
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        let source_cell = self
            .owner
            .branch_cell(source_branch)
            .ok_or(RelationalForkDenial::SourceBranchMissing)?;
        let source_snapshot = source_cell.atomic_snapshot();
        match source_snapshot.lifecycle_posture() {
            super::RelationalBranchLifecyclePosture::Live => {}
            super::RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalForkDenial::SourceArchived);
            }
            super::RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalForkDenial::SourceDeleting);
            }
        }
        let observation = source_snapshot.observation();
        let truth_version = source_snapshot.truth_version();
        self.owner.record_lookup();
        if observation.target().is_empty() {
            return Err(RelationalForkDenial::EmptySource);
        }
        let descriptor = RelationalForkSourceDescriptor::new(
            self.runtime_instance_id,
            observation,
            source_branch.clone(),
            truth_version,
        );
        let token = admit_relational_fork_source(descriptor.clone());
        Ok((descriptor, token))
    }

    /// Consume a fork-only source token and create a fresh branch reference.
    /// The immutable source artifact is shared by identity; no envelope copy
    /// or source-cell mutation is performed.
    pub fn fork_branch(
        &self,
        target_branch: BranchId,
        source: AdmittedRelationalForkSourceBasis,
    ) -> Result<RelationalForkOutcome, RelationalForkDenial> {
        let reservation = self.reserve_fork_target(target_branch)?;
        self.fork_reserved(reservation, source)
    }

    /// Reserve one exact destination without installing or moving a branch.
    /// The returned custody is consumed by [`Self::fork_reserved`] or releases
    /// the reservation exactly once when dropped.
    pub fn reserve_fork_target(
        &self,
        target_branch: BranchId,
    ) -> Result<super::RelationalForkTargetReservation, RelationalForkDenial> {
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        self.owner
            .reserve_target(target_branch)
            .map_err(|denial| match denial {
                super::RelationalForkTargetReservationDenial::Duplicate => {
                    RelationalForkDenial::DuplicateTarget
                }
                super::RelationalForkTargetReservationDenial::Retired => {
                    RelationalForkDenial::RetiredTarget
                }
            })
    }

    /// Consume owner-issued destination custody and source evidence through
    /// the canonical fork authority path.
    pub fn fork_reserved(
        &self,
        reservation: super::RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
    ) -> Result<RelationalForkOutcome, RelationalForkDenial> {
        self.fork_reserved_with_hooks(reservation, source, || {}, || {}, false)
            .map(|(outcome, _)| outcome)
    }

    /// Consume owner-issued fork custody and return the authentic admitted
    /// basis of the installed destination in the same owner operation.
    pub fn fork_reserved_with_basis(
        &self,
        reservation: super::RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
    ) -> Result<(RelationalForkOutcome, super::AdmittedRelationalBranchBasis), RelationalForkDenial>
    {
        self.fork_reserved_with_hooks(reservation, source, || {}, || {}, true)
            .map(|(outcome, basis)| {
                (
                    outcome,
                    basis.expect("basis-enabled fork installation returns its basis"),
                )
            })
    }

    #[cfg(test)]
    pub(crate) fn fork_branch_with_test_pause(
        &self,
        target_branch: BranchId,
        source: AdmittedRelationalForkSourceBasis,
        reached: &std::sync::Barrier,
        release: &std::sync::Barrier,
    ) -> Result<RelationalForkOutcome, RelationalForkDenial> {
        let reservation = self.reserve_fork_target(target_branch)?;
        self.fork_reserved_with_hooks(
            reservation,
            source,
            || {
                reached.wait();
                release.wait();
            },
            || {},
            false,
        )
        .map(|(outcome, _)| outcome)
    }

    #[cfg(test)]
    pub(crate) fn fork_branch_with_post_install_test_pause(
        &self,
        target_branch: BranchId,
        source: AdmittedRelationalForkSourceBasis,
        reached: &std::sync::Barrier,
        release: &std::sync::Barrier,
    ) -> Result<(RelationalForkOutcome, super::AdmittedRelationalBranchBasis), RelationalForkDenial>
    {
        let reservation = self.reserve_fork_target(target_branch)?;
        self.fork_reserved_with_hooks(
            reservation,
            source,
            || {},
            || {
                reached.wait();
                release.wait();
            },
            true,
        )
        .map(|(outcome, basis)| {
            (
                outcome,
                basis.expect("basis-enabled fork installation returns its basis"),
            )
        })
    }

    fn fork_reserved_with_hooks(
        &self,
        reservation: super::RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
        pause_before_install: impl FnOnce(),
        pause_after_install: impl FnOnce(),
        return_basis: bool,
    ) -> Result<
        (
            RelationalForkOutcome,
            Option<super::AdmittedRelationalBranchBasis>,
        ),
        RelationalForkDenial,
    > {
        if !self.owner.owns_reservation(&reservation) {
            return Err(RelationalForkDenial::ForeignRuntime);
        }
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        let target_branch = reservation.branch_id().clone();
        let (descriptor, _authority) = source.into_parts();
        let validated = self.validate_fork_source(descriptor)?;
        pause_before_install();
        let prepared = self.prepare_fork_target(target_branch, reservation, &validated)?;
        let installed = self.install_fork_target(validated, prepared, return_basis)?;
        pause_after_install();
        Ok(installed)
    }

    fn validate_fork_source(
        &self,
        descriptor: RelationalForkSourceDescriptor,
    ) -> Result<ValidatedForkSource, RelationalForkDenial> {
        if descriptor.runtime_instance_id() != self.runtime_instance_id {
            return Err(RelationalForkDenial::ForeignRuntime);
        }
        let (source_observation, source_truth_version, source_root) = {
            let source_cell = self
                .owner
                .branch_cell(descriptor.source_branch())
                .ok_or(RelationalForkDenial::SourceBranchMissing)?;
            let snapshot = source_cell.atomic_snapshot();
            match snapshot.lifecycle_posture() {
                super::RelationalBranchLifecyclePosture::Live => {}
                super::RelationalBranchLifecyclePosture::Archived => {
                    return Err(RelationalForkDenial::SourceArchived);
                }
                super::RelationalBranchLifecyclePosture::Deleting => {
                    return Err(RelationalForkDenial::SourceDeleting);
                }
            }
            (
                snapshot.observation(),
                snapshot.truth_version(),
                snapshot.root(),
            )
        };
        self.owner.record_lookup();
        if descriptor.truth_version() != source_truth_version
            || descriptor
                .observation()
                .compare(&source_observation)
                .is_err()
        {
            return Err(RelationalForkDenial::StaleSource);
        }
        Ok(ValidatedForkSource {
            descriptor,
            source_observation,
            source_truth_version,
            source_root: source_root.ok_or(RelationalForkDenial::MissingArtifact)?,
        })
    }

    fn prepare_fork_target(
        &self,
        target_branch: BranchId,
        reservation: super::RelationalForkTargetReservation,
        source: &ValidatedForkSource,
    ) -> Result<PreparedForkTarget, RelationalForkDenial> {
        let source_root = Arc::clone(&source.source_root);
        let target_cell = crate::branch::RelationalBranchReferenceCell::from_source_with_root(
            self.runtime_instance_id,
            target_branch,
            source.descriptor.source_branch().clone(),
            &source.source_observation,
            source_root.clone(),
        )
        .map_err(|_| RelationalForkDenial::InvalidTarget(BranchCreateError::invalid_target()))?;
        let target_observation = target_cell.observation().clone();
        let fork_provenance = target_cell
            .fork_provenance()
            .expect("forked branch must retain exact source provenance");
        let target_truth_version = target_cell.truth_version();
        let shared_commit_id = match target_observation.target() {
            FoundationalBranchTarget::Empty => None,
            FoundationalBranchTarget::Basis(target) => Some(CommitId(target.selected_commit_id())),
        };
        let source_head_version = if let Some(commit_id) = shared_commit_id {
            let version_id = source_root
                .canonical_envelope()
                .filter(|envelope| envelope.commit.commit_id == commit_id)
                .map(|envelope| envelope.commit.version_id)
                .ok_or(RelationalForkDenial::MissingArtifact)?;
            Some(version_id)
        } else {
            None
        };
        Ok(PreparedForkTarget {
            target_cell,
            source_root,
            target_observation,
            fork_provenance,
            target_truth_version,
            shared_commit_id,
            source_head_version,
            reservation,
        })
    }
}

struct ValidatedForkSource {
    descriptor: RelationalForkSourceDescriptor,
    source_observation: RelationalBranchReferenceObservation,
    source_truth_version: RelationalBranchVersion,
    source_root: Arc<crate::branch::RelationalBranchRoot>,
}

struct PreparedForkTarget {
    target_cell: crate::branch::RelationalBranchReferenceCell,
    source_root: Arc<crate::branch::RelationalBranchRoot>,
    target_observation: RelationalBranchReferenceObservation,
    fork_provenance: RelationalBranchReferenceObservation,
    target_truth_version: RelationalBranchVersion,
    shared_commit_id: Option<CommitId>,
    source_head_version: Option<crate::identity::data::VersionId>,
    reservation: super::RelationalForkTargetReservation,
}

#[path = "fork_runtime_adapters.rs"]
mod runtime_adapters;

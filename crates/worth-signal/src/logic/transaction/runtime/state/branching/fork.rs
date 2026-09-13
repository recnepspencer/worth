use worth_proof::TransitionOutcome;

use crate::data::error::SignalError;
use crate::state::{SignalBranchHandle, SignalSnapshotId, SignalSnapshotV1};

use super::super::runtime_state::{ExplicitBranchForkPacket, SignalRuntime};
use super::branches::{BranchAncestryState, BranchState};
use super::fork_snapshot::materialize_snapshot_fork_state;
use super::fork_validation::{expect_fork_branch_basis, validate_fork_branch_name};
use super::{
    SignalBranchBasisArtifact, SignalBranchForkDenial, SignalBranchForkReceipt,
    SignalBranchForkRequest, SignalBranchForkRequestBasis,
};

struct ResolvedForkRequest<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    parent_branch: SignalBranchHandle,
    parent_basis: SignalBranchBasisArtifact,
    requested_snapshot_basis: Option<SignalBranchBasisArtifact>,
    created_branch_head_snapshot_id: Option<SignalSnapshotId>,
    source_branch_state: BranchState<D, I, T>,
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn fork_branch(
        &mut self,
        request: SignalBranchForkRequest,
    ) -> TransitionOutcome<SignalBranchForkReceipt, SignalBranchForkDenial> {
        if matches!(
            request.basis(),
            SignalBranchForkRequestBasis::ParentBranchSnapshot { .. }
        ) {
            self.with_telemetry(|telemetry| telemetry.transaction.explicit_fork_denial_count += 1);
            return TransitionOutcome::denied(
                SignalBranchForkDenial::SnapshotPayloadRequiredForFork {
                    request: request.clone(),
                },
            );
        }
        self.fork_branch_resolved(request, None)
    }

    pub(crate) fn fork_branch_with_snapshot(
        &mut self,
        request: SignalBranchForkRequest,
        snapshot: &SignalSnapshotV1,
    ) -> TransitionOutcome<SignalBranchForkReceipt, SignalBranchForkDenial> {
        self.fork_branch_resolved(request, Some(snapshot))
    }

    fn fork_branch_resolved(
        &mut self,
        request: SignalBranchForkRequest,
        snapshot: Option<&SignalSnapshotV1>,
    ) -> TransitionOutcome<SignalBranchForkReceipt, SignalBranchForkDenial> {
        let validated_name = match validate_fork_branch_name(request.branch_name()) {
            Ok(name) => name,
            Err(_) => {
                self.with_telemetry(|telemetry| {
                    telemetry.transaction.explicit_fork_denial_count += 1
                });
                return TransitionOutcome::denied(SignalBranchForkDenial::InvalidBranchIdentity);
            }
        };
        let resolved = match self.resolve_branch_fork_request(&request, snapshot) {
            Ok(resolved) => resolved,
            Err(denial) => {
                self.with_telemetry(|telemetry| {
                    telemetry.transaction.explicit_fork_denial_count += 1
                });
                return TransitionOutcome::denied(denial);
            }
        };

        let current_branch_name = resolved.parent_branch.name.clone();
        let parent_branch_id = resolved.parent_branch.id;
        let mut branch_state = resolved.source_branch_state;
        let handle = match self.branches.create_live_branch(
            validated_name,
            parent_branch_id,
            resolved.created_branch_head_snapshot_id,
        ) {
            Ok(handle) => handle,
            Err(_) => {
                return TransitionOutcome::denied(SignalBranchForkDenial::BranchIdentityExhausted)
            }
        };
        *branch_state.ancestry_mut() = BranchAncestryState::new(
            handle.id,
            Some(parent_branch_id),
            resolved.created_branch_head_snapshot_id,
        );
        branch_state.reset_mutation_ledger(resolved.created_branch_head_snapshot_id);
        branch_state.clear_branch_mutation_nodes();
        self.branches
            .branch_mutation_ledger_mut(parent_branch_id, resolved.parent_branch.head_snapshot_id)
            .clear_all(resolved.parent_branch.head_snapshot_id);
        self.branches
            .project_catalog(handle.id, branch_state.graph_mut());
        self.with_telemetry(|telemetry| telemetry.transaction.explicit_fork_count += 1);
        if resolved.requested_snapshot_basis.is_some() {
            self.with_telemetry(|telemetry| {
                telemetry.transaction.explicit_snapshot_fork_count += 1
            });
        }
        self.branches
            .store_fork_packet(ExplicitBranchForkPacket::new(
                parent_branch_id,
                handle.id,
                branch_state,
            ))
            .expect("validated fork admission must produce a well-formed fork packet");
        self.project_branch_catalog();
        crate::diagnostics::recorder::record_snapshot_event(
            &mut self.graph,
            crate::diagnostics::replay::ReplayEventKind::BranchCreated,
            None,
            format!("created branch `{}`", handle.name),
        );
        crate::diagnostics::recorder::record_branch_fork_lineage(
            &mut self.graph,
            handle.id,
            parent_branch_id,
            handle.name.clone(),
            current_branch_name,
        );

        let created_branch_basis = match self.branch_basis_artifact(handle.clone()) {
            TransitionOutcome::Success(basis) => basis,
            other => {
                panic!("created branch basis must validate immediately after admission: {other:?}")
            }
        };
        let active_branch_after_fork_basis = self.current_branch_basis_artifact();

        TransitionOutcome::success(SignalBranchForkReceipt {
            request,
            parent_basis: resolved.parent_basis,
            requested_snapshot_basis: resolved.requested_snapshot_basis,
            created_branch: handle,
            created_branch_basis,
            active_branch_after_fork_basis,
        })
    }

    fn resolve_branch_fork_request(
        &mut self,
        request: &SignalBranchForkRequest,
        snapshot: Option<&SignalSnapshotV1>,
    ) -> Result<ResolvedForkRequest<D, I, T>, SignalBranchForkDenial> {
        match request.basis() {
            SignalBranchForkRequestBasis::CurrentBranchHead => {
                let parent_branch = self.graph.current_branch();
                Ok(ResolvedForkRequest {
                    parent_basis: self.current_branch_basis_artifact(),
                    created_branch_head_snapshot_id: parent_branch.head_snapshot_id,
                    requested_snapshot_basis: None,
                    source_branch_state: self
                        .capture_heavy_branch_state()
                        .map_err(Self::branch_transfer_error_to_fork_denial)?,
                    parent_branch,
                })
            }
            SignalBranchForkRequestBasis::ParentBranchHead { parent_branch_id } => {
                let parent_branch = self.branch_handle(*parent_branch_id).ok_or(
                    SignalBranchForkDenial::UnknownParentBranch {
                        parent_branch_id: *parent_branch_id,
                    },
                )?;
                let parent_basis =
                    expect_fork_branch_basis(self.branch_basis_artifact(parent_branch.clone()));
                let source_branch_state =
                    self.materialize_parent_head_fork_state(parent_branch.clone())?;
                Ok(ResolvedForkRequest {
                    created_branch_head_snapshot_id: parent_branch.head_snapshot_id,
                    parent_basis,
                    requested_snapshot_basis: None,
                    source_branch_state,
                    parent_branch,
                })
            }
            SignalBranchForkRequestBasis::ParentBranchSnapshot {
                parent_branch_id,
                snapshot_id,
            } => {
                let snapshot = snapshot.ok_or_else(|| {
                    SignalBranchForkDenial::SnapshotPayloadRequiredForFork {
                        request: request.clone(),
                    }
                })?;
                let parent_branch = self.branch_handle(*parent_branch_id).ok_or(
                    SignalBranchForkDenial::UnknownParentBranch {
                        parent_branch_id: *parent_branch_id,
                    },
                )?;
                if snapshot.meta.snapshot_id != *snapshot_id {
                    return Err(SignalBranchForkDenial::SnapshotBasisMismatch {
                        requested_snapshot_id: *snapshot_id,
                        provided_snapshot_id: snapshot.meta.snapshot_id,
                    });
                }
                if snapshot.meta.branch_id != parent_branch.id {
                    return Err(SignalBranchForkDenial::IncompatibleForkSnapshotLineage {
                        parent_branch_id: parent_branch.id,
                        snapshot_branch_id: snapshot.meta.branch_id,
                        snapshot_id: snapshot.meta.snapshot_id,
                    });
                }
                if self
                    .branches
                    .snapshot_state(snapshot.meta.branch_id, snapshot.meta.snapshot_id)
                    .is_none()
                {
                    return Err(SignalBranchForkDenial::UnknownForkSnapshot {
                        parent_branch_id: parent_branch.id,
                        snapshot_id: snapshot.meta.snapshot_id,
                    });
                }
                let parent_basis =
                    expect_fork_branch_basis(self.branch_basis_artifact(parent_branch.clone()));
                let requested_snapshot_basis = expect_fork_branch_basis(
                    self.snapshot_branch_basis_artifact(parent_branch.clone(), snapshot),
                );
                let source_branch_state =
                    materialize_snapshot_fork_state(self, parent_branch.clone(), snapshot)?;
                Ok(ResolvedForkRequest {
                    created_branch_head_snapshot_id: Some(snapshot.meta.snapshot_id),
                    parent_basis,
                    requested_snapshot_basis: Some(requested_snapshot_basis),
                    source_branch_state,
                    parent_branch,
                })
            }
        }
    }

    fn materialize_parent_head_fork_state(
        &mut self,
        parent_branch: SignalBranchHandle,
    ) -> Result<BranchState<D, I, T>, SignalBranchForkDenial> {
        if parent_branch.id == self.graph.current_branch().id {
            return self
                .capture_heavy_branch_state()
                .map_err(Self::branch_transfer_error_to_fork_denial);
        }
        let state = self.branches.branch_state(parent_branch.id).ok_or(
            SignalBranchForkDenial::UnknownParentBranch {
                parent_branch_id: parent_branch.id,
            },
        )?;
        Self::ensure_managed_queue_branch_transfer_allowed(state.resource())
            .map_err(Self::branch_transfer_error_to_fork_denial)?;
        Ok(state.clone())
    }

    pub(super) fn branch_transfer_error_to_fork_denial(
        error: SignalError,
    ) -> SignalBranchForkDenial {
        match error {
            SignalError::ManagedQueueBranchTransferDenied { bound_queue_count } => {
                SignalBranchForkDenial::ManagedQueueBranchTransferDenied { bound_queue_count }
            }
            other => panic!("heavy branch capture returned an undeclared failure: {other}"),
        }
    }
}

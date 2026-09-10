use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdmissionLease, SignalBranchAdvanceDenial,
    SignalBranchAdvanceEngineDenial, SignalBranchAdvanceOutcome,
};
use crate::data::error::SignalError;
use crate::logic::transaction::TransactionResult;
use crate::state::SignalBranchHandle;

use super::super::runtime_state::SignalRuntime;
use super::{BranchTargetedTransactionDenial, BranchTargetedTransactionRequest};

struct SignalBranchAdvancePreflight {
    branch: SignalBranchHandle,
    retention: SignalBranchAdmissionLease,
    lane: SignalBranchAdvanceLane,
}

enum SignalBranchAdvanceLane {
    Active,
    Stored,
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn advance_signal_branch<F>(
        &mut self,
        runtime_ctx: &mut Ctx,
        expected: &AdmittedSignalBranchBasis,
        apply: F,
    ) -> Result<SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>
    where
        F: FnOnce(
            &mut crate::logic::transaction::runtime::transaction::SignalTransaction<
                '_,
                D,
                I,
                E,
                Ctx,
                T,
            >,
        ) -> Result<(), SignalError>,
    {
        if let Some((_, mutation, _)) = self.sealed_owner_port_slots() {
            let cancellation = crate::branch::SignalOwnerCancellationSource::new();
            return mutation.advance_exact(expected, runtime_ctx, &cancellation.token(), apply);
        }
        let preflight = self.preflight_signal_branch_advance(expected)?;
        let transaction = self.execute_signal_branch_advance(runtime_ctx, &preflight, apply)?;
        let advanced_basis = self
            .admit_signal_branch_with_retention(preflight.branch, preflight.retention)
            .expect("retained live branch must remain admissible after canonical advance");
        Ok(SignalBranchAdvanceOutcome::owner_issued(
            advanced_basis,
            transaction,
            None,
        ))
    }

    fn preflight_signal_branch_advance(
        &self,
        expected: &AdmittedSignalBranchBasis,
    ) -> Result<SignalBranchAdvancePreflight, SignalBranchAdvanceDenial> {
        let branch_id = expected.owner_branch_id();
        let branch = self
            .branches
            .branch_handle(branch_id)
            .ok_or(SignalBranchAdvanceDenial::UnknownBranch { branch_id })?;
        let live = self
            .signal_branch_observation(&branch)
            .map_err(|_| SignalBranchAdvanceDenial::UnknownBranch { branch_id })?;
        if let Err(mismatch) = live.compare(expected.observation()) {
            return Err(SignalBranchAdvanceDenial::BasisMismatch {
                axes: mismatch.axes().to_vec(),
            });
        }
        let retention = self
            .branches
            .acquire_admitted_retention(branch_id)
            .map_err(|denial| SignalBranchAdvanceDenial::RetentionUnavailable { denial })?;
        let lane = if branch_id == self.graph.current_branch().id {
            SignalBranchAdvanceLane::Active
        } else {
            SignalBranchAdvanceLane::Stored
        };
        Ok(SignalBranchAdvancePreflight {
            branch,
            retention,
            lane,
        })
    }

    fn execute_signal_branch_advance<F>(
        &mut self,
        runtime_ctx: &mut Ctx,
        preflight: &SignalBranchAdvancePreflight,
        apply: F,
    ) -> Result<TransactionResult, SignalBranchAdvanceDenial>
    where
        F: FnOnce(
            &mut crate::logic::transaction::runtime::transaction::SignalTransaction<
                '_,
                D,
                I,
                E,
                Ctx,
                T,
            >,
        ) -> Result<(), SignalError>,
    {
        match preflight.lane {
            SignalBranchAdvanceLane::Active => self
                .transaction(runtime_ctx, apply)
                .map_err(|error| SignalBranchAdvanceDenial::MutationFailedNoMovement { error }),
            SignalBranchAdvanceLane::Stored => self.execute_stored_signal_branch_advance(
                runtime_ctx,
                preflight.branch.clone(),
                apply,
            ),
        }
    }

    fn execute_stored_signal_branch_advance<F>(
        &mut self,
        runtime_ctx: &mut Ctx,
        branch: SignalBranchHandle,
        apply: F,
    ) -> Result<TransactionResult, SignalBranchAdvanceDenial>
    where
        F: FnOnce(
            &mut crate::logic::transaction::runtime::transaction::SignalTransaction<
                '_,
                D,
                I,
                E,
                Ctx,
                T,
            >,
        ) -> Result<(), SignalError>,
    {
        let branch_id = branch.id;
        let head = self
            .branch_transaction_head(branch.clone())
            .into_result()
            .map_err(advance_engine_denial)?;
        let plan = self
            .plan_branch_targeted_transaction(BranchTargetedTransactionRequest::new(branch, head))
            .into_result()
            .map_err(advance_engine_denial)?;
        match self.execute_branch_targeted_transaction(runtime_ctx, plan, apply) {
            TransitionOutcome::Success(receipt) => Ok(receipt.transaction().clone()),
            TransitionOutcome::Denied(denial) => Err(advance_engine_denial(denial)),
            TransitionOutcome::Failed(error) => {
                Err(SignalBranchAdvanceDenial::MutationFailedNoMovement { error })
            }
            _ => Err(SignalBranchAdvanceDenial::MutationDeniedNoMovement {
                denial: SignalBranchAdvanceEngineDenial::UnknownTargetBranch { branch_id },
            }),
        }
    }
}

fn advance_engine_denial(denial: BranchTargetedTransactionDenial) -> SignalBranchAdvanceDenial {
    SignalBranchAdvanceDenial::MutationDeniedNoMovement {
        denial: map_targeted_denial(denial),
    }
}

fn map_targeted_denial(denial: BranchTargetedTransactionDenial) -> SignalBranchAdvanceEngineDenial {
    match denial {
        BranchTargetedTransactionDenial::UnknownTargetBranch { branch_id } => {
            SignalBranchAdvanceEngineDenial::UnknownTargetBranch { branch_id }
        }
        BranchTargetedTransactionDenial::ActiveBranchTarget { branch_id } => {
            SignalBranchAdvanceEngineDenial::ActiveBranchTarget { branch_id }
        }
        BranchTargetedTransactionDenial::CrossBranchHead {
            target_branch_id,
            head_branch_id,
        } => SignalBranchAdvanceEngineDenial::CrossBranchHead {
            target_branch_id,
            head_branch_id,
        },
        BranchTargetedTransactionDenial::StaleTargetHead { .. } => {
            SignalBranchAdvanceEngineDenial::StaleTargetHead
        }
        BranchTargetedTransactionDenial::CanonicalBasisMismatch => {
            SignalBranchAdvanceEngineDenial::CanonicalBasisMismatch
        }
    }
}

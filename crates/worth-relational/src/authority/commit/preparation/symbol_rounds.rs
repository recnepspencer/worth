//! A symbol miss abandons a candidate before any bucket load can occur.
use crate::authority::mutation::MutationApplyDenial;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::runtime::{
    PendingSymbolMeanings, RelationalPreparationRuntime, SymbolReservationDenial,
};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::transactions::data::{CommitPreparationError, TransactionCommitError};

pub(crate) fn probe_symbol_rounds<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    mut round: impl FnMut(
        PendingSymbolMeanings,
        &RelationalPreparationRuntime,
    ) -> Result<T, MutationApplyDenial>,
) -> Result<T, TransactionCommitError> {
    let mut admitted_buckets = [false; 256];
    loop {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::PreparationRound,
        )?;
        let effects = runtime.preparation_round();
        let answer = round(runtime.services.symbols.begin_pending_meanings(), &effects);
        if !matches!(
            &answer,
            Err(MutationApplyDenial::Symbols(
                SymbolReservationDenial::ColdBucket(_)
            ))
        ) {
            runtime.settle_preparation_round(&effects);
        }
        drop(effects);
        match answer {
            Ok(accepted) => return Ok(accepted),
            Err(MutationApplyDenial::Commit(error)) => return Err(error),
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::IdentityExhausted)) => {
                return Err(TransactionCommitError::preparation(
                    CommitPreparationError::symbol_identity_exhausted(
                        basis.identity().branch_id().clone(),
                        basis.observation().version_id(),
                    ),
                ));
            }
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::StorageUnavailable(
                failure,
            ))) => {
                return Err(super::storage_read_denial(basis, failure.into()));
            }
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::ColdBucket(text))) => {
                use sha2::{Digest, Sha256};
                let bucket = Sha256::digest(text.as_str().as_bytes())[0] as usize;
                if std::mem::replace(&mut admitted_buckets[bucket], true) {
                    return Err(super::storage_read_denial(
                        basis,
                        RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Symbols {
                                first: crate::symbols::data::Symbol(0),
                                last: crate::symbols::data::Symbol(u32::MAX - 1),
                            },
                            cause: RelationalStorageUnavailableCause::Damaged,
                        }
                        .into(),
                    ));
                }
                // The round's owned draft and reservations have already dropped.
                runtime
                    .services
                    .symbols
                    .demand_text(execution, text.as_str())
                    .map_err(|failure| super::storage_read_denial(basis, failure))?;
            }
        }
    }
}

//! A symbol miss abandons a candidate before any bucket load can occur.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::{PendingSymbolMeanings, SymbolReservationDenial};
use contracts::storage::data::storage_unavailable::RelationalStorageSubject;
use contracts::storage::data::storage_unavailable::RelationalStorageUnavailable;
use contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause;
use contracts::transactions::data::outcomes::commit_preparation::CommitPreparationError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;

use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

#[derive(Default)]
pub struct PreparationRoundObservations {
    pub instrumentation: RuntimeInstrumentation,
}

impl PreparationRoundObservations {
    pub fn new(instrumentation: &RuntimeInstrumentation) -> Self {
        Self {
            instrumentation: instrumentation.preparation_round(),
        }
    }

    pub fn settle(
        instrumentation: &RuntimeInstrumentation,
        round_instrumentation: &RuntimeInstrumentation,
    ) {
        let actual = round_instrumentation.complexity_snapshot();
        instrumentation.count(|counters| {
            contracts::performance::operation_complexity_accounting::accumulate_complexity_counts(
                counters, actual,
            );
        });
    }
}

pub struct SymbolRoundInputs<'a, R> {
    pub source: &'a R,
    pub symbols: &'a RuntimeSymbolTable,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub bind: fn(&R, &PreparationRoundObservations) -> R,
}

pub fn probe_symbol_rounds<'a, R: 'a, T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<SymbolRoundInputs<'a, R>>,
    basis: &AdmittedRelationalBranchBasis,
    mut round: impl FnMut(PendingSymbolMeanings, &R) -> Result<T, MutationApplyDenial>,
) -> Result<T, TransactionCommitError> {
    // There are exactly 256 symbol buckets. Each cold miss admits a previously
    // unseen bucket, so at most 256 retries (257 attempts) are possible. A
    // repeated miss returns typed StorageUnavailable(Damaged) below, rather
    // than permitting another retry with no progress (D1619).
    let inputs = inputs.into();
    let mut admitted_buckets = [false; 256];
    loop {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::PreparationRound,
        )?;
        contracts::execution::phase_observation::symbol_preparation_event(
            contracts::execution::phase_observation::SymbolPreparationEvent::RoundStarted,
        );
        let observations = PreparationRoundObservations::new(inputs.instrumentation);
        let scoped = (inputs.bind)(inputs.source, &observations);
        let answer = round(inputs.symbols.begin_pending_meanings(), &scoped);
        if !matches!(
            &answer,
            Err(MutationApplyDenial::Symbols(
                SymbolReservationDenial::ColdBucket(_)
            ))
        ) {
            PreparationRoundObservations::settle(
                inputs.instrumentation,
                &observations.instrumentation,
            );
        }
        drop(scoped);
        drop(observations);
        match answer {
            Ok(accepted) => {
                contracts::execution::phase_observation::symbol_preparation_event(
                    contracts::execution::phase_observation::SymbolPreparationEvent::ProbeReturnedOk,
                );
                return Ok(accepted);
            }
            Err(MutationApplyDenial::Commit(error)) => return Err(error),
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::InvalidDictionary)) => {
                return Err(TransactionCommitError::publication(
                    contracts::publication::data::publication_error::PublicationError::new(
                        contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                        "candidate dictionary conflicts with reserved symbol meaning",
                    ),
                ));
            }
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
                return Err(worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(basis, failure.into()));
            }
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::ColdBucket(text))) => {
                contracts::execution::phase_observation::symbol_preparation_event(
                    contracts::execution::phase_observation::SymbolPreparationEvent::ColdBucketRetry,
                );
                use sha2::{Digest, Sha256};
                let bucket = Sha256::digest(text.as_str().as_bytes())[0] as usize;
                if std::mem::replace(&mut admitted_buckets[bucket], true) {
                    return Err(worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(
                        basis,
                        RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Symbols {
                                first: contracts::symbols::data::symbol::Symbol(0),
                                last: contracts::symbols::data::symbol::Symbol(u32::MAX - 1),
                            },
                            cause: RelationalStorageUnavailableCause::Damaged,
                        }
                        .into(),
                    ));
                }
                // The round's owned draft and reservations have already dropped.
                inputs
                    .symbols
                    .demand_text(execution, text.as_str())
                    .map_err(|failure| worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(basis, failure))?;
            }
        }
    }
}

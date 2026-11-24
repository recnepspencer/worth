//! A symbol miss abandons a candidate before any bucket load can occur.
use crate::authority::mutation::MutationApplyDenial;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings;
use crate::runtime::state::subsystems::services::symbol_table::SymbolReservationDenial;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::transactions::data::outcomes::commit_preparation::CommitPreparationError;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;

use crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

#[derive(Default)]
pub(crate) struct PreparationRoundObservations {
    pub(crate) instrumentation: RuntimeInstrumentation,
    pub(crate) diagnostics: RelationalDiagnosticArtifactStore,
}

impl PreparationRoundObservations {
    pub(crate) fn new(instrumentation: &RuntimeInstrumentation) -> Self {
        Self {
            instrumentation: instrumentation.preparation_round(),
            diagnostics: Default::default(),
        }
    }

    pub(crate) fn settle(
        instrumentation: &RuntimeInstrumentation,
        diagnostics: &RelationalDiagnosticArtifactStore,
        round_instrumentation: &RuntimeInstrumentation,
        round_diagnostics: &RelationalDiagnosticArtifactStore,
    ) {
        let actual = round_instrumentation.complexity_snapshot();
        instrumentation.count(|counters| {
            crate::performance::operation_complexity_accounting::accumulate_complexity_counts(
                counters, actual,
            );
        });
        for artifact in round_diagnostics.snapshot() {
            diagnostics.push(artifact);
        }
    }
}

pub(crate) struct SymbolRoundInputs<'a, R> {
    pub(crate) source: &'a R,
    pub(crate) symbols: &'a RuntimeSymbolTable,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
    pub(crate) diagnostics: &'a RelationalDiagnosticArtifactStore,
    pub(crate) bind: fn(&R, &PreparationRoundObservations) -> R,
}

pub(crate) fn probe_symbol_rounds<'a, R: 'a, T>(
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
        crate::execution::phase_observation::symbol_preparation_event(
            crate::execution::phase_observation::SymbolPreparationEvent::RoundStarted,
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
                inputs.diagnostics,
                &observations.instrumentation,
                &observations.diagnostics,
            );
        }
        drop(scoped);
        drop(observations);
        match answer {
            Ok(accepted) => {
                crate::execution::phase_observation::symbol_preparation_event(
                    crate::execution::phase_observation::SymbolPreparationEvent::ProbeReturnedOk,
                );
                return Ok(accepted);
            }
            Err(MutationApplyDenial::Commit(error)) => return Err(error),
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::InvalidDictionary)) => {
                return Err(TransactionCommitError::publication(
                    crate::publication::data::PublicationError::new(
                        crate::publication::bundle::PublicationStage::BundleAssembly,
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
                return Err(super::storage_read_denial(basis, failure.into()));
            }
            Err(MutationApplyDenial::Symbols(SymbolReservationDenial::ColdBucket(text))) => {
                crate::execution::phase_observation::symbol_preparation_event(
                    crate::execution::phase_observation::SymbolPreparationEvent::ColdBucketRetry,
                );
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
                inputs
                    .symbols
                    .demand_text(execution, text.as_str())
                    .map_err(|failure| super::storage_read_denial(basis, failure))?;
            }
        }
    }
}

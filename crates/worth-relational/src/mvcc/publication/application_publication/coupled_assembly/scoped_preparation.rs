//! Scoped Query dispatch completes the real Native draft before final judging.
use super::*;
use crate::branch::{SelectedRelationalBranchRead, SelectedRelationalBranchState};
use crate::mvcc::{RelationalPreparationPort, RelationalTransactionValidationInput};

// Both issued forms retain their actual Native context for the duration of the
// call. Only the existing issuer constructs either form; no identity is minted here.
trait IssuedProposal {
    fn identity(&self) -> &RelationalMutationProposalIdentity;
}
impl IssuedProposal for RelationalMutationProposalIdentity {
    fn identity(&self) -> &RelationalMutationProposalIdentity {
        self
    }
}
impl<Context> IssuedProposal for (RelationalMutationProposalIdentity, Context) {
    fn identity(&self) -> &RelationalMutationProposalIdentity {
        &self.0
    }
}

impl RelationalPreparationPort {
    /// Run the installed Query assembly against the actual unpublished prefix.
    /// The callback is borrowed only for this call, and is never retained by a
    /// transaction, history, a candidate registration, or the publication owner.
    pub fn prepare_authored_coupled_transaction(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        bound: RelationalAuthoredCoupledTransaction,
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<Option<BranchBoundRelationalTransaction>, TransactionCommitError> {
        let mut transaction = bound.transaction;
        let configuration = self.binding.configuration_binding();
        let epoch = configuration.operation();
        let runtime = self
            .binding
            .runtime_snapshot_from(&epoch)
            .with_authored_coupling(transaction.authored_coupling.clone())
            .with_commit_work_budget(None);
        let _operation = runtime.admit_operation().ok_or_else(|| {
            part::reject(
                crate::publication::data::AuthoredRetirementRefusal::ForeignCoupledOwner,
                transaction
                    .authored_coupling
                    .as_ref()
                    .expect("bound assembly")
                    .owners
                    .source
                    .subject(),
            )
        })?;
        let basis = transaction.basis.clone();
        transaction.ensure_current_basis(&runtime)?;
        transaction.demand_preparation_inputs(
            execution,
            &runtime.services.symbols,
            &runtime.index_definitions,
        )?;
        let selected = SelectedRelationalBranchState::from_admitted_basis(&basis);
        let complete = crate::storage::residency::probe_head(
            execution,
            &selected.with_symbol_admission(&runtime.services.symbols),
            |partitions| {
                let selected = SelectedRelationalBranchRead::in_round(&selected, partitions);
                crate::authority::commit::preparation::probe_symbol_rounds(
                    execution,
                    &runtime,
                    &basis,
                    |mut symbols, round| {
                        transaction.with_accepted_preparation_overlay(|transaction| {
                            if let Some(prior) = &transaction.pending_symbols {
                                symbols.adopt(prior)?;
                            }
                            transaction
                                .validate_staged_branch_locality(&selected, symbols.interner())?;
                            transaction.reserve_preparation_client_keys(
                                &mut symbols,
                                runtime.config.identity.client_key_symbol_policy,
                                execution.allocation_policy(),
                            )?;
                            let input =
                                RelationalTransactionValidationInput::from_transaction(transaction);
                            let issued = runtime.issue_mutation_proposal_identity(
                                transaction.transaction_id,
                                &input,
                            )?;
                            let complete = super::completion::prepare_coupled_candidate(
                                execution,
                                round,
                                transaction,
                                selected,
                                issued.identity(),
                                &mut symbols,
                                assembly,
                            )?;
                            crate::execution::phase_observation::symbol_preparation_event(
                                crate::execution::phase_observation::SymbolPreparationEvent::CoupledCompleted(complete),
                            );
                            if !complete {
                                return Ok(false);
                            }
                            let binding = transaction
                                .authored_coupling
                                .as_ref()
                                .expect("scoped preparation retains binding");
                            let mut prepared =
                                binding.applied.lock().unwrap_or_else(|e| e.into_inner());
                            let prepared = prepared.as_mut().expect("assembly completed its draft");
                            std::mem::swap(&mut symbols, &mut prepared.mutation.identities.symbols);
                            Ok(true)
                        })
                    },
                )
            },
        )
        .map_err(|cause| {
            crate::authority::commit::preparation::storage_read_denial(&basis, cause)
        })??;
        Ok(complete.then_some(transaction))
    }
}

impl BoundAuthoredCoupling {
    pub(crate) fn inherit_prepared_symbols(
        &self,
        symbols: &mut crate::runtime::PendingSymbolMeanings,
    ) -> Result<(), crate::authority::mutation::MutationApplyDenial> {
        let prepared = self.applied.lock().unwrap_or_else(|e| e.into_inner());
        let prepared = prepared.as_ref().ok_or_else(|| {
            part::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.owners.source.subject(),
            )
        })?;
        symbols.adopt(prepared.mutation.candidate_symbols())?;
        Ok(())
    }
    pub(crate) fn adopt_prepared_symbols(
        &self,
        identity: &RelationalMutationProposalIdentity,
        symbols: &mut crate::runtime::PendingSymbolMeanings,
    ) -> Result<(), TransactionCommitError> {
        let mut prepared = self.applied.lock().unwrap_or_else(|e| e.into_inner());
        let prepared = prepared.as_mut().ok_or_else(|| {
            part::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.owners.source.subject(),
            )
        })?;
        if !prepared.identity.same_proposal_sequence(identity) {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::ForeignCoupledOwner,
                self.owners.source.subject(),
            ));
        }
        std::mem::swap(symbols, &mut prepared.mutation.identities.symbols);
        Ok(())
    }
}

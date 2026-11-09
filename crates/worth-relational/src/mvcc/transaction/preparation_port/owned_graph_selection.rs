//! Only an actual retained graph can enter this dedicated preparation receiver.
use super::*;
use crate::mvcc::owned_graph_selection::RelationalOwnedGraphSelectionPreparationError as Error;
use crate::{
    branch::RelationalOwnedModelGraphSelection,
    mvcc::{
        owned_graph_selection::*, RelationalInstalledApplicationPublicationAuthority,
        RequiredOutputPublication,
    },
};
use std::sync::Arc;
impl RelationalPreparationPort {
    pub fn admit_owned_graph_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        graph: RelationalOwnedModelGraphSelection,
    ) -> Result<RelationalOwnedGraphSelection, Error> {
        let runtime = self.binding.runtime_snapshot();
        let _operation = runtime.admit_operation().ok_or(Error::OwnerUnavailable)?;
        runtime
            .require_retained_selection_profile()
            .map_err(|_| Error::DurabilityUnsupported {
                model: graph.entities().model(),
            })?;
        selection_admission::require_pristine(&transaction)
            .map_err(|_| Error::TargetNotPristine)?;
        if graph.entities().current_basis().descriptor() != transaction.basis.descriptor() {
            return Err(Error::BasisChanged {
                model: graph.entities().model(),
            });
        }
        runtime
            .validate_retained_selection_sources(
                &transaction.basis,
                graph.entities().selected_basis(),
            )
            .map_err(Error::Source)?;
        let count = graph.boundary_entities().count()
            + graph.current_relations().len()
            + graph.selected_relations().len();
        let installed = execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, count as u64)?;
                graph.ownership_matches_installation(&runtime, context)
            })
            .map_err(Error::Request)?
            .0
            .map_err(|stop| Error::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        if !installed {
            return Err(Error::OwnershipNotInstalled {
                model: graph.entities().model(),
            });
        }
        let funding = execution
            .reserve_retained_memory((count as u64).saturating_mul(128))
            .map_err(Error::Request)?;
        let meaning = crate::history::data::CanonicalOwnedGraphSelection::from_prepared(&graph)
            .map_err(Error::Preparation)?;
        Ok(RelationalOwnedGraphSelection {
            transaction,
            retained: PreparedOwnedGraphSelection {
                graph: Arc::new(graph),
                meaning: Arc::new(meaning),
                _funding: Arc::new(funding),
            },
        })
    }
    pub fn prepare_owned_graph_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        input: RelationalOwnedGraphSelection,
        publication: Option<(
            &RelationalInstalledApplicationPublicationAuthority,
            RequiredOutputPublication,
        )>,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, Error> {
        let configuration = self.binding.configuration_binding();
        let epoch = configuration.operation();
        let runtime = self.binding.runtime_snapshot_from(&epoch);
        let _operation = runtime.admit_operation().ok_or(Error::OwnerUnavailable)?;
        runtime
            .require_retained_selection_profile()
            .map_err(|_| Error::DurabilityUnsupported {
                model: input.retained.graph.entities().model(),
            })?;
        runtime
            .validate_retained_selection_sources(
                &input.transaction.basis,
                input.retained.graph.entities().selected_basis(),
            )
            .map_err(Error::Source)?;
        crate::mvcc::validation::require_not_interrupted(
            &input.transaction.control,
            &input.transaction.basis.inner.retention_binding,
            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
        )
        .map_err(Error::Preparation)?;
        let runtime = if execution.is_leased() {
            runtime.with_commit_work_budget(None)
        } else {
            runtime
        };
        let installed = execution
            .run_native_with_checkpoint(|context| {
                input
                    .retained
                    .graph
                    .ownership_matches_installation(&runtime, context)
            })
            .map_err(Error::Request)?
            .0
            .map_err(|stop| Error::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        if !installed {
            return Err(Error::OwnershipNotInstalled {
                model: input.retained.graph.entities().model(),
            });
        }
        admit_selected_symbols(execution, &runtime, &input.retained)?;
        crate::mvcc::require_graph_coverage(
            execution,
            &input.retained.graph,
            publication.as_ref().map(|(_, publication)| publication),
        )
        .map_err(Error::Preparation)?;
        let mut proposal = runtime
            .validate_consulted_owned_graph_selection(execution, input.transaction, input.retained)
            .map_err(Error::Preparation)?;
        if let Some((installed, publication)) = publication {
            proposal = proposal
                .bind_required_output_publication(installed, publication)
                .map_err(Error::Preparation)?;
        }
        self.prepare_validated_proposal_inner(execution, &runtime, proposal)
            .map_err(Error::Preparation)
    }
}

fn admit_selected_symbols(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    retained: &PreparedOwnedGraphSelection,
) -> Result<(), Error> {
    let basis = retained.graph.entities().selected_basis();
    let historical = crate::branch::SelectedRelationalBranchState::from_admitted_basis(basis);
    crate::storage::residency::probe_head(
        execution,
        &historical.with_symbol_admission(&runtime.services.symbols),
        |source| {
            for &id in retained.graph.restored_entities() {
                source.get_current_partition(id.partition_id);
            }
            for id in retained.graph.selected_relations().keys() {
                source.get_current_partition(id.partition_id);
            }
        },
    )
    .map_err(|failure| {
        Error::Preparation(crate::authority::commit::preparation::storage_read_denial(
            basis, failure,
        ))
    })
}

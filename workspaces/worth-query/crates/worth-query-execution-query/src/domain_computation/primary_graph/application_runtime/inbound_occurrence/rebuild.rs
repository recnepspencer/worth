//! Explicit finite repair of the disposable terminal correlation index.

use std::num::NonZeroUsize;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

use super::reconstruct::pair_world_history;
use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};
use crate::domain_computation::primary_graph::provider::{
    inbound_terminal_index::InboundTerminalSource, WorthQueryCanonicalInboundCompletion,
    WorthQueryInboundCompletionReadDenial, WorthQueryInboundTerminalIndexDenial as Denial,
};

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn rebuild_completed_inbound_index(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        maximum_world_slots: NonZeroUsize,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<bool, Denial> {
        let page = self
            .runtime
            .primary_provider
            .inbound_reconstruction_page(maximum_world_slots)?;
        let mut entries = Vec::new();
        for (correlation, source) in page.sources {
            match self.rebuild_terminal_from_source(
                execution,
                &source,
                maximum_changed_records,
                maximum_ancestry_commits,
            ) {
                Ok(entry) if entry.correlation() == &correlation => entries.push(entry),
                Ok(_) => {
                    self.runtime
                        .primary_provider
                        .abandon_inbound_reconstruction();
                    return Err(Denial::WorldPairMismatch);
                }
                Err(denial) => {
                    self.runtime
                        .primary_provider
                        .abandon_inbound_reconstruction();
                    return Err(denial);
                }
            }
        }
        self.runtime
            .primary_provider
            .advance_inbound_reconstruction(page.generation, page.next, entries)
    }

    pub fn rebuild_terminal_from_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &InboundTerminalSource,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<WorthQueryCanonicalInboundCompletion, Denial> {
        let observation = self.runtime.product_runtime.owner_ref().observation_port();
        match source {
            InboundTerminalSource::Performed { .. } => {
                let inspection = self.runtime.product_runtime.owner_ref().inspection_port();
                let history = inspection
                    .trace_ancestry(source.commit_identity().clone(), maximum_ancestry_commits)
                    .map_err(|_| Denial::IndexUnavailable)?;
                let commit = history.commits().next().ok_or(Denial::WorldPairMismatch)?;
                let relational = commit
                    .relational_publication_identity()
                    .ok_or(Denial::WorldPairMismatch)?;
                let row = self
                    .runtime
                    .primary_provider
                    .completion_row_from_commit_patch(
                        execution,
                        relational,
                        maximum_changed_records,
                    )
                    .map_err(completion_read_denial)?
                    .ok_or(Denial::WorldPairMismatch)?;
                match pair_world_history(&history, &row) {
                    Ok((original, completion, attempt)) => {
                        if &completion != source.commit_identity()
                            || row.original_incarnation_ordinal != source.incarnation().ordinal()
                            || commit.relational_change()
                                != worth_runtime_world::facade::CompositeComponentChangePosture::Published
                        {
                            return Err(Denial::WorldPairMismatch);
                        }
                        let record = self
                            .runtime
                            .primary_provider
                            .completion_original_outbox(
                                execution,
                                &row,
                                &original,
                                source.incarnation(),
                                &observation,
                                maximum_changed_records,
                            )
                            .map_err(completion_read_denial)?;
                        let original_receipt = row.original_commit.clone();
                        let original_world = original.clone();
                        let sealed = WorthQueryCanonicalInboundCompletion::from_verified_row(
                            row,
                            source.incarnation(),
                            original,
                            completion,
                            attempt,
                        );
                        if !sealed.matches_original_dispatch(
                            &record,
                            &original_receipt,
                            &original_world,
                            source.incarnation(),
                        ) {
                            return Err(Denial::WorldPairMismatch);
                        }
                        Ok(sealed)
                    }
                    Err(Denial::IndexUnavailable)
                        if history.crossed_retired_history()
                            && source
                                .commit_identity()
                                .owner_identity()
                                .durable()
                                .is_some() =>
                    {
                        let worth_runtime_world::facade::CompositeCommitProvenance::Publication(
                            attempt,
                        ) = commit.provenance()
                        else {
                            return Err(Denial::WorldPairMismatch);
                        };
                        if commit.identity() != source.commit_identity()
                            || commit.relational_change()
                                != worth_runtime_world::facade::CompositeComponentChangePosture::Published
                            || relational.commit_id() != row.completion_commit.commit_id
                            || relational.version_id() != row.completion_commit.version_id
                            || relational.authoring_branch() != &row.completion_commit.branch_id
                            || row.original_incarnation_ordinal != source.incarnation().ordinal()
                        {
                            return Err(Denial::WorldPairMismatch);
                        }
                        let completion_world = self
                            .runtime
                            .primary_provider
                            .observe_home_publication_for_receipt(
                                execution,
                                &row.completion_commit,
                                &observation,
                            )
                            .map_err(completion_read_denial)?;
                        if completion_world.composite_commit() != source.commit_identity()
                            || completion_world.product_incarnation() != source.incarnation()
                            || completion_world.publication_attempt() != attempt
                            || completion_world.native_receipt() != &row.completion_commit
                        {
                            return Err(Denial::WorldPairMismatch);
                        }
                        self.pair_home_completion_source(
                            execution,
                            &completion_world,
                            row,
                            &observation,
                            maximum_changed_records,
                            maximum_ancestry_commits,
                        )
                    }
                    Err(denial) => Err(denial),
                }
            }
            InboundTerminalSource::Readmitted { world, entity } => {
                let row = self
                    .runtime
                    .primary_provider
                    .completion_row_from_readmitted_source(execution, world, *entity)
                    .map_err(completion_read_denial)?;
                self.pair_home_completion_source(
                    execution,
                    world,
                    row,
                    &observation,
                    maximum_changed_records,
                    maximum_ancestry_commits,
                )
            }
        }
    }

    fn pair_home_completion_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &worth_runtime_world::facade::WorldObservedHomePublication,
        row: crate::domain_computation::primary_graph::provider::WorthQueryCanonicalCompletionRow,
        observation: &worth_runtime_world::facade::RuntimeWorldObservationPort,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<WorthQueryCanonicalInboundCompletion, Denial> {
        if &row.completion_commit != world.native_receipt()
            || row.original_incarnation_ordinal != world.product_incarnation().ordinal()
        {
            return Err(Denial::WorldPairMismatch);
        }
        let original_world = self
            .runtime
            .primary_provider
            .observe_home_publication_for_receipt(execution, &row.original_commit, observation)
            .map_err(completion_read_denial)?;
        if original_world.after().branch().owner() != world.after().branch().owner()
            || original_world.product_branch() != world.product_branch()
            || original_world.product_incarnation() != world.product_incarnation()
            || original_world.native_receipt() != &row.original_commit
            || original_world.product_generation().get() >= world.product_generation().get()
        {
            return Err(Denial::WorldPairMismatch);
        }
        let span = world.product_generation().get() - original_world.product_generation().get();
        if span >= maximum_ancestry_commits.get() as u64 {
            return Err(Denial::ReconstructionWorkExhausted);
        }
        let original_commit = original_world.composite_commit().clone();
        let record = self
            .runtime
            .primary_provider
            .completion_original_outbox_from_observed(
                execution,
                original_world,
                &row,
                maximum_changed_records,
            )
            .map_err(completion_read_denial)?;
        let original_receipt = row.original_commit.clone();
        let sealed = WorthQueryCanonicalInboundCompletion::from_verified_row(
            row,
            world.product_incarnation(),
            original_commit.clone(),
            world.composite_commit().clone(),
            world.publication_attempt().clone(),
        );
        if !sealed.matches_original_dispatch(
            &record,
            &original_receipt,
            &original_commit,
            world.product_incarnation(),
        ) {
            return Err(Denial::WorldPairMismatch);
        }
        Ok(sealed)
    }
}

pub fn completion_read_denial(denial: WorthQueryInboundCompletionReadDenial) -> Denial {
    match denial {
        WorthQueryInboundCompletionReadDenial::SourceRead(cause) => Denial::SourceRead(cause),
        WorthQueryInboundCompletionReadDenial::ReadRequest(cause) => Denial::ReadRequest(cause),
        WorthQueryInboundCompletionReadDenial::ReconstructionWorkExhausted => {
            Denial::ReconstructionWorkExhausted
        }
        WorthQueryInboundCompletionReadDenial::AmbiguousCorrelation
        | WorthQueryInboundCompletionReadDenial::Malformed
        | WorthQueryInboundCompletionReadDenial::CommitUnavailable => Denial::IndexUnavailable,
    }
}

/// Explicit repair result; ordinary exact lookup never invokes this lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundIndexRepairDenial {
    SourceRead(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    ExecutionDenied(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    ForeignPhase,
    ForeignVerifier,
    IndexUnavailable,
    ReconstructionWorkExhausted,
    WorldPairMismatch,
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Inspect one owner-wide World page under three finite work axes. The
    /// installed route caps page width; changed-record and ancestry budgets
    /// are explicit repair-only ceilings and may need to exceed ordinary
    /// discovery work for a long-lived branch.
    /// `Ok(false)` retains the cursor; until `Ok(true)`, exact terminal lookup
    /// remains unavailable. Reconstruction is separate from ordinary discovery.
    pub fn repair_completed_inbound_index(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_world_slots: NonZeroUsize,
        maximum_changed_records_per_commit: NonZeroUsize,
        maximum_ancestry_commits_per_completion: NonZeroUsize,
    ) -> Result<bool, WorthQueryInboundIndexRepairDenial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| WorthQueryInboundIndexRepairDenial::ForeignPhase)?;
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(WorthQueryInboundIndexRepairDenial::ForeignVerifier)?;
        let installed_page_limit =
            usize::try_from(installed.contract.limits().maximum_discovery_work.get())
                .unwrap_or(usize::MAX);
        let maximum_world_slots =
            NonZeroUsize::new(maximum_world_slots.get().min(installed_page_limit))
                .expect("installed discovery work is nonzero");
        self.rebuild_completed_inbound_index(
            &execution,
            maximum_world_slots,
            maximum_changed_records_per_commit,
            maximum_ancestry_commits_per_completion,
        )
        .map_err(|denial| match denial {
            Denial::SourceRead(cause) => WorthQueryInboundIndexRepairDenial::SourceRead(cause),
            Denial::ReadRequest(cause) => WorthQueryInboundIndexRepairDenial::ExecutionDenied(
                crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None),
            ),
            Denial::IndexUnavailable | Denial::NotExpired => {
                WorthQueryInboundIndexRepairDenial::IndexUnavailable
            }
            Denial::ReconstructionWorkExhausted => {
                WorthQueryInboundIndexRepairDenial::ReconstructionWorkExhausted
            }
            Denial::WorldPairMismatch => WorthQueryInboundIndexRepairDenial::WorldPairMismatch,
        })
    }
}

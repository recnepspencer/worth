//! Explicit finite repair of the disposable terminal correlation index.

use std::num::NonZeroUsize;

use super::reconstruct::pair_world_history;
use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};
use crate::domain_computation::primary_graph::provider::{
    inbound_terminal_index::InboundTerminalSource, WorthQueryCanonicalInboundCompletion,
    WorthQueryInboundCompletionReadDenial, WorthQueryInboundTerminalIndexDenial as Denial,
};

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(in crate::domain_computation) fn rebuild_completed_inbound_index(
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

    pub(in crate::domain_computation::primary_graph::application_runtime) fn rebuild_terminal_from_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &InboundTerminalSource,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<WorthQueryCanonicalInboundCompletion, Denial> {
        let inspection = self.runtime.product_runtime.owner.inspection_port();
        let history = inspection
            .trace_ancestry(source.commit_identity().clone(), maximum_ancestry_commits)
            .map_err(|_| Denial::IndexUnavailable)?;
        let commit = history.commits().next().ok_or(Denial::WorldPairMismatch)?;
        let row = match source {
            InboundTerminalSource::Performed { .. } => {
                let relational = commit
                    .relational_publication_identity()
                    .ok_or(Denial::WorldPairMismatch)?;
                self.runtime
                    .primary_provider
                    .completion_row_from_commit_patch(
                        execution,
                        relational,
                        maximum_changed_records,
                    )
                    .map_err(completion_read_denial)?
                    .ok_or(Denial::WorldPairMismatch)?
            }
            InboundTerminalSource::Readmitted { world, entity } => self
                .runtime
                .primary_provider
                .completion_row_from_readmitted_source(execution, world, *entity)
                .map_err(completion_read_denial)?,
        };
        let (original, completion, attempt) = pair_world_history(&history, &row)?;
        if &completion != source.commit_identity()
            || row.original_incarnation_ordinal != source.incarnation().ordinal()
        {
            return Err(Denial::WorldPairMismatch);
        }
        if commit.relational_change()
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
                &self.runtime.product_runtime.owner.observation_port(),
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
}

pub(super) fn completion_read_denial(denial: WorthQueryInboundCompletionReadDenial) -> Denial {
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

//! Explicit finite repair of the disposable terminal correlation index.

use crate::domain_computation::primary_graph::application_inbound::InboundOccurrenceScopedOps;
use std::num::NonZeroUsize;

use worth_runtime_world::facade::{CompositeComponentChangePosture, RuntimeWorldPublicationRow};

use super::reconstruct::pair_world_history;
use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryCanonicalInboundCompletion, WorthQueryInboundCompletionReadDenial,
    WorthQueryInboundTerminalIndexDenial as Denial,
};

pub(in crate::domain_computation) trait RebuildScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn rebuild_completed_inbound_index(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        maximum_world_slots: NonZeroUsize,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<bool, Denial> {
        let (cursor, generation) = self.primary_provider().inbound_reconstruction_cursor();
        let inspection = self.product_runtime().owner.inspection_port();
        let page = inspection
            .performed_publication_page(cursor.as_ref(), maximum_world_slots)
            .map_err(|_| {
                self.primary_provider().abandon_inbound_reconstruction();
                Denial::IndexUnavailable
            })?;
        if page.pending() {
            self.primary_provider().abandon_inbound_reconstruction();
            return Err(Denial::IndexUnavailable);
        }
        let mut entries = Vec::new();
        for world in page.rows() {
            match self.rebuild_one_terminal(
                execution,
                world,
                maximum_changed_records,
                maximum_ancestry_commits,
            ) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => {}
                Err(denial) => {
                    self.primary_provider().abandon_inbound_reconstruction();
                    return Err(denial);
                }
            }
        }
        if page.next_after().is_none()
            && !inspection
                .publication_frontier_is_current(page.frontier())
                .map_err(|_| Denial::IndexUnavailable)?
        {
            self.primary_provider().abandon_inbound_reconstruction();
            return Err(Denial::IndexUnavailable);
        }
        self.primary_provider().advance_inbound_reconstruction(
            generation,
            page.frontier(),
            page.next_after().cloned(),
            entries,
        )
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> RebuildScopedOps<Schema> for HostRuntime {}

trait RebuildOwnOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn rebuild_one_terminal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        world: &RuntimeWorldPublicationRow,
        maximum_changed_records: NonZeroUsize,
        maximum_ancestry_commits: NonZeroUsize,
    ) -> Result<Option<WorthQueryCanonicalInboundCompletion>, Denial> {
        let Some(relational) = world.commit().relational_publication_identity() else {
            return Ok(None);
        };
        let row = self
            .primary_provider()
            .completion_row_from_commit_patch(execution, relational, maximum_changed_records)
            .map_err(|denial| match denial {
                WorthQueryInboundCompletionReadDenial::ReadRequest(cause) => {
                    Denial::ReadRequest(cause)
                }
                WorthQueryInboundCompletionReadDenial::ReconstructionWorkExhausted => {
                    Denial::ReconstructionWorkExhausted
                }
                _ => Denial::IndexUnavailable,
            })?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.original_incarnation_ordinal != world.product_incarnation().ordinal()
            || world.commit().relational_change() != CompositeComponentChangePosture::Published
            || world
                .component_results()
                .relational_commit_result()
                .is_none()
            || world.component_results().relational_settlement() != Some(&row.completion_commit)
        {
            return Err(Denial::WorldPairMismatch);
        }
        let history = self
            .product_runtime()
            .owner
            .inspection_port()
            .trace_ancestry(world.commit().identity().clone(), maximum_ancestry_commits)
            .map_err(|_| Denial::IndexUnavailable)?;
        let (original, completion, attempt) = pair_world_history(&history, &row)?;
        if &completion != world.commit().identity() || &attempt != world.publication_attempt() {
            return Err(Denial::WorldPairMismatch);
        }
        Ok(Some(
            WorthQueryCanonicalInboundCompletion::from_verified_row(
                row,
                world.product_incarnation(),
                original,
                completion,
                attempt,
            ),
        ))
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> RebuildOwnOps<Schema> for HostRuntime {}

/// Explicit repair result; ordinary exact lookup never invokes this lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundIndexRepairDenial {
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

pub trait RebuildOps<Schema: worth_query_installation::facade::ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn repair_completed_inbound_index(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_world_slots: NonZeroUsize,
        maximum_changed_records_per_commit: NonZeroUsize,
        maximum_ancestry_commits_per_completion: NonZeroUsize,
    ) -> Result<bool, WorthQueryInboundIndexRepairDenial> {
        let execution = phase
            .execution_request_for(self.product_runtime())
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

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> RebuildOps<Schema> for HostRuntime {}

//! Continue one actual unpublished completion without synthesizing a Performed carrier.
use crate::domain_computation::primary_graph::provider::{
    inbound_terminal_index::InboundTerminalSource, WorthQueryCanonicalInboundCompletion,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductUnpublishedRecoveryHandle, WorldRecoveredContinuationOutcome,
    WorldRecoveredUnpublishedEffects,
};
mod release;
use crate::domain_computation::execution_runtime::product_world::{
    OwnerCleanupRetirements, WorthQueryProductBranchOwnerCleanup,
};
use crate::domain_computation::primary_graph::home_original_progression;

use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::inbound_publication::recovered::OriginalCompletionState;

pub enum RecordedCompletionProgress {
    Completed(WorthQueryCanonicalInboundCompletion),
    StaleReleased,
}

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::recorded_completion_denial::WorthQueryRecordedCompletionDenial;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::inbound_publication::recovered::RecordedCompletionContinuation;

pub trait RecordedCompletionContinuationAdvanceExt {
    fn advance<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<RecordedCompletionProgress, std::sync::Arc<WorthQueryRecordedCompletionDenial>>;
}

impl RecordedCompletionContinuationAdvanceExt for RecordedCompletionContinuation {
    fn advance<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<RecordedCompletionProgress, std::sync::Arc<WorthQueryRecordedCompletionDenial>>
    {
        let result = advance_original(self, runtime, execution, retirements);
        match result {
            Ok(progress) => {
                self.last_denial = None;
                Ok(progress)
            }
            Err(cause) => {
                let cause = std::sync::Arc::new(cause);
                self.last_denial = Some(cause.clone());
                Err(cause)
            }
        }
    }
}

fn advance_original<Schema: worth_query_installation::facade::ApplicationSchema>(
    continuation: &mut RecordedCompletionContinuation,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    retirements: &mut OwnerCleanupRetirements,
) -> Result<RecordedCompletionProgress, WorthQueryRecordedCompletionDenial> {
    use release::RecordedCompletionContinuationReleaseExt as _;
    use WorthQueryRecordedCompletionDenial as Denial;
    match &continuation.state {
        OriginalCompletionState::Releasing
        | OriginalCompletionState::Cleanup(_)
        | OriginalCompletionState::Released => {
            return continuation.release_stale(runtime, execution, retirements)
        }
        OriginalCompletionState::Unpublished | OriginalCompletionState::Published(_) => {}
    }
    if matches!(continuation.state, OriginalCompletionState::Unpublished) {
        let recovery = runtime.runtime.product_runtime.owner.recovery_port();
        let effects = recovery
            .inspect_recovered_effects(&continuation.handle)
            .map_err(Denial::World)?;
        match home_original_progression::continue_original(&recovery, execution, effects) {
            WorldRecoveredContinuationOutcome::Published { commit } => {
                continuation.state = OriginalCompletionState::Published(commit)
            }
            WorldRecoveredContinuationOutcome::Pending(_) => return Err(Denial::Pending),
            WorldRecoveredContinuationOutcome::Refused {
                cause:
                    worth_runtime_world::facade::WorldRecoveredContinuationDenial::PredecessorChanged,
                custody,
            } => {
                drop(custody);
                continuation.state = OriginalCompletionState::Releasing;
                return continuation.release_stale(runtime, execution, retirements);
            }
            WorldRecoveredContinuationOutcome::Refused { cause, .. } => {
                return Err(Denial::Refused(cause))
            }
            WorldRecoveredContinuationOutcome::EvidenceRequired(_) => {
                return Err(Denial::EvidenceRequired)
            }
            WorldRecoveredContinuationOutcome::Indeterminate(_) => {
                return Err(Denial::Indeterminate)
            }
        }
    }
    let world = runtime
        .runtime
        .primary_provider
        .completion_source_from_subject(
            execution,
            &continuation.subject,
            &runtime.runtime.product_runtime.owner.observation_port(),
        )
        .map_err(Denial::Source)?;
    let OriginalCompletionState::Published(commit) = &continuation.state else {
        unreachable!("original continued or release returned above")
    };
    if world.after().selected_commit() != commit {
        return Err(Denial::PublicationMismatch);
    }
    let source = InboundTerminalSource::Readmitted {
        world,
        entity: continuation.entity,
    };
    let bound = std::num::NonZeroUsize::new(usize::MAX)
        .expect("cold request controls source traversal");
    let terminal = runtime
        .rebuild_terminal_from_source(execution, &source, bound, bound)
        .map_err(Denial::Terminal)?;
    let InboundTerminalSource::Readmitted { world, entity } = source else {
        unreachable!("actual readmitted source")
    };
    runtime
        .runtime
        .primary_provider
        .retain_readmitted_inbound_terminal(terminal.clone(), world, entity)
        .map_err(Denial::Terminal)?;
    runtime
        .runtime
        .primary_provider
        .clear_inbound_completion_publication_pending(terminal.correlation());
    Ok(RecordedCompletionProgress::Completed(terminal))
}

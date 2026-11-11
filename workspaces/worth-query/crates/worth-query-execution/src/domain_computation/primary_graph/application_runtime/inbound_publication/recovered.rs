//! Continue one actual unpublished completion without synthesizing a Performed carrier.
use crate::domain_computation::primary_graph::provider::{
    inbound_terminal_index::InboundTerminalSource, WorthQueryCanonicalInboundCompletion,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductUnpublishedRecoveryHandle, WorldRecoveredContinuationOutcome,
    WorldRecoveredUnpublishedEffects,
};
mod denial;
mod release;
use crate::domain_computation::execution_runtime::product_world::{
    OwnerCleanupRetirements, WorthQueryProductBranchOwnerCleanup,
};
use crate::domain_computation::primary_graph::home_original_progression;

#[derive(Clone)]
enum OriginalCompletionState {
    Unpublished,
    Releasing,
    Cleanup(WorthQueryProductBranchOwnerCleanup),
    Released,
    Published(CompositeCommitIdentity),
}

pub(in crate::domain_computation::primary_graph) enum RecordedCompletionProgress {
    Completed(WorthQueryCanonicalInboundCompletion),
    StaleReleased,
}

#[derive(Debug)]
pub enum WorthQueryRecordedCompletionDenial {
    World(worth_runtime_world::facade::RuntimeWorldRecoveryDenial),
    Pending,
    Refused(worth_runtime_world::facade::WorldRecoveredContinuationDenial),
    EvidenceRequired,
    Indeterminate,
    Source(
        crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionReadDenial,
    ),
    Terminal(
        crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial,
    ),
    PublicationMismatch,
}

#[derive(Clone)]
pub(in crate::domain_computation) struct RecordedCompletionContinuation {
    handle: ProductUnpublishedRecoveryHandle,
    subject: worth_relational::facade::durability::RelationalAssociatedEffectSubject,
    entity: worth_relational::facade::identity::EntityId,
    state: OriginalCompletionState,
    last_denial: Option<std::sync::Arc<WorthQueryRecordedCompletionDenial>>,
}

impl RecordedCompletionContinuation {
    pub(in crate::domain_computation) fn same_original(&self, other: &Self) -> bool {
        self.handle == other.handle
            && self.subject.receipt() == other.subject.receipt()
            && self.subject.position() == other.subject.position()
            && self.subject.artifact_digest() == other.subject.artifact_digest()
            && self.entity == other.entity
    }
    pub(in crate::domain_computation) fn from_original(
        effects: &WorldRecoveredUnpublishedEffects,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Option<Self> {
        Some(Self {
            handle: effects.recovery_handle(),
            subject: effects.native_effect()?.subject().clone(),
            entity,
            state: OriginalCompletionState::Unpublished,
            last_denial: None,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn advance<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<RecordedCompletionProgress, std::sync::Arc<WorthQueryRecordedCompletionDenial>>
    {
        let result = self.advance_original(runtime, execution, retirements);
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

    pub(in crate::domain_computation) fn last_denial(
        &self,
    ) -> Option<std::sync::Arc<WorthQueryRecordedCompletionDenial>> {
        self.last_denial.clone()
    }

    fn advance_original<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<RecordedCompletionProgress, WorthQueryRecordedCompletionDenial> {
        use WorthQueryRecordedCompletionDenial as Denial;
        match &self.state {
            OriginalCompletionState::Releasing
            | OriginalCompletionState::Cleanup(_)
            | OriginalCompletionState::Released => {
                return self.release_stale(runtime, execution, retirements)
            }
            OriginalCompletionState::Unpublished | OriginalCompletionState::Published(_) => {}
        }
        if matches!(self.state, OriginalCompletionState::Unpublished) {
            let recovery = runtime.runtime.product_runtime.owner.recovery_port();
            let effects = recovery
                .inspect_recovered_effects(&self.handle)
                .map_err(Denial::World)?;
            match home_original_progression::continue_original(&recovery, execution, effects) {
                WorldRecoveredContinuationOutcome::Published { commit } => {
                    self.state = OriginalCompletionState::Published(commit)
                }
                WorldRecoveredContinuationOutcome::Pending(_) => return Err(Denial::Pending),
                WorldRecoveredContinuationOutcome::Refused {
                    cause: worth_runtime_world::facade::WorldRecoveredContinuationDenial::PredecessorChanged,
                    custody,
                } => {
                    drop(custody);
                    self.state = OriginalCompletionState::Releasing;
                    return self.release_stale(runtime, execution, retirements);
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
                &self.subject,
                &runtime.runtime.product_runtime.owner.observation_port(),
            )
            .map_err(Denial::Source)?;
        let OriginalCompletionState::Published(commit) = &self.state else {
            unreachable!("original continued or release returned above")
        };
        if world.after().selected_commit() != commit {
            return Err(Denial::PublicationMismatch);
        }
        let source = InboundTerminalSource::Readmitted {
            world,
            entity: self.entity,
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
}

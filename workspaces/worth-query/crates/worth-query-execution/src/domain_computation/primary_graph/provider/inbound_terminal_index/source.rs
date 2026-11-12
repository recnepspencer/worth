//! Canonical completion protection retains its actual live or readmitted source.
use super::*;
use std::sync::Arc;
use worth_runtime_world::facade::WorldReadmittedPublication;

pub(in crate::domain_computation::primary_graph) enum InboundTerminalSource {
    Performed {
        protection: RuntimeWorldPerformedPublicationProtection,
        incarnation: ProductBranchIncarnation,
    },
    Readmitted {
        world: WorldReadmittedPublication,
        entity: worth_relational::facade::identity::EntityId,
    },
}
impl InboundTerminalSource {
    pub(in crate::domain_computation::primary_graph) fn incarnation(
        &self,
    ) -> ProductBranchIncarnation {
        match self {
            Self::Performed { incarnation, .. } => *incarnation,
            Self::Readmitted { world, .. } => world.after().lifecycle_incarnation(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn commit_identity(
        &self,
    ) -> &CompositeCommitIdentity {
        match self {
            Self::Performed { protection, .. } => protection.commit_identity(),
            Self::Readmitted { world, .. } => world.after().selected_commit(),
        }
    }
}
impl WorthQueryInboundTerminalIndex {
    pub(in crate::domain_computation::primary_graph::provider) fn advance_source_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial,
    > {
        use crate::domain_computation::primary_graph::{
            WorthQueryHomeRetirementDenial as Denial,
            WorthQueryOperationHomeRetirementProgress as Progress,
        };
        request.consult().map_err(Denial::Execution)?;
        let state = self.state.get_mut().unwrap_or_else(|p| p.into_inner());
        state.rebuilding = None;
        if state.protected.pop_first().is_some() {
            Ok(Progress::Advanced)
        } else {
            Ok(Progress::Complete)
        }
    }
    pub(super) fn retain_readmitted(
        &self,
        sealed: WorthQueryCanonicalInboundCompletion,
        world: WorldReadmittedPublication,
        entity: worth_relational::facade::identity::EntityId,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        use WorthQueryInboundTerminalIndexDenial as Denial;
        if world.after().selected_commit() != sealed.completion_world_commit()
            || world.attempt_identity() != sealed.completion_attempt()
            || world.native_receipt() != &sealed.completion_relational_commit
        {
            return Err(Denial::WorldPairMismatch);
        }
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(existing) = state.protected.get(sealed.correlation()) {
            return if existing.commit_identity() == sealed.completion_world_commit() {
                Ok(())
            } else {
                Err(Denial::WorldPairMismatch)
            };
        }
        if let Some(derived) = &mut state.derived {
            derived.insert(sealed.correlation, sealed.clone());
        }
        state.protected.insert(
            sealed.correlation,
            Arc::new(InboundTerminalSource::Readmitted { world, entity }),
        );
        state.generation = state.generation.wrapping_add(1);
        state.rebuilding = None;
        Ok(())
    }
}

pub(in crate::domain_computation::primary_graph) struct InboundTerminalSourcePage {
    pub generation: u64,
    pub sources: Vec<(
        ExternalEffectCorrelationIdentity,
        Arc<InboundTerminalSource>,
    )>,
    pub next: Option<ExternalEffectCorrelationIdentity>,
}

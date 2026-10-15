//! Governed data values. No view or historical actor grants command authority.
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::ReadOnlyWorkflowHistory;
use crate::domain_computation::primary_graph::application_attempt::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
use std::sync::Arc;
use worth_execution::ExecutionArray;
use worth_relational::facade::identity::EntityId;

/// Complete historical identity, disclosed only by independently admitted audit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowRecordedActor(EntityId);
impl WorthQueryWorkflowRecordedActor {
    pub(in crate::domain_computation::primary_graph) fn observed(entity: EntityId) -> Self {
        Self(entity)
    }
    /// Recorded identity, with generation preserved; it carries no permission.
    pub fn entity_id(&self) -> EntityId {
        self.0
    }
}
/// Recorded roles of one transition at the selected observation.
pub struct WorthQueryWorkflowTransitionActors {
    pub(super) advancer: WorthQueryWorkflowRecordedActor,
    pub(super) executor: Option<WorthQueryWorkflowRecordedActor>,
}
impl WorthQueryWorkflowTransitionActors {
    pub(in crate::domain_computation::primary_graph) fn observed(
        advancer: WorthQueryWorkflowRecordedActor,
        executor: Option<WorthQueryWorkflowRecordedActor>,
    ) -> Self {
        Self { advancer, executor }
    }

    pub fn advancer(&self) -> WorthQueryWorkflowRecordedActor {
        self.advancer
    }
    pub fn executor(&self) -> Option<WorthQueryWorkflowRecordedActor> {
        self.executor
    }
}
/// Independent audit may withhold identities without hiding lifecycle state.
pub enum WorthQueryWorkflowAudit {
    Withheld,
    Visible {
        start: WorthQueryWorkflowRecordedActor,
        end: Option<WorthQueryWorkflowRecordedActor>,
        transitions: ExecutionArray<WorthQueryWorkflowTransitionActors>,
    },
}
/// Recorded instance state, independent of whether a command is currently legal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowObservedState {
    Ready,
    Completed,
    Cancelled,
    Migrated,
}
/// Why the discovered record is descriptive history rather than a live reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowHistoricalPosture {
    RetainedBasis,
    Fork,
    Ended,
    Unlinked,
}
/// Exact read outcome. Only live discovery yields a command locator, never permission.
pub enum WorthQueryWorkflowInstanceRead {
    Missing {
        basis: WorthQueryWorkflowReadBasis,
    },
    Live {
        instance: PublishedWorkflowInstanceRef,
        observation: WorthQueryWorkflowObservation,
    },
    Historical {
        posture: WorthQueryWorkflowHistoricalPosture,
        observation: WorthQueryWorkflowObservation,
    },
}
/// Owns the exact data basis even when no instance exists; carries no admission.
pub struct WorthQueryWorkflowReadBasis(Arc<WorthQueryApplicationReadObservation>);
impl WorthQueryWorkflowReadBasis {
    pub(super) fn retained<Schema: worth_query_installation::facade::ApplicationSchema>(
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Self {
        // Retain only the native read lease for the result's observer lifetime.
        // The selected operation's publication source and fresh admission stay private.
        Self(WorthQueryApplicationReadObservation::from_product(
            selected.application(),
            selected.product().read_lease(),
        ))
    }
    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.0.selected_commit()
    }
    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.0.branch_identity()
    }
}
/// State and bounded history at the explicit retained data observation.
/// This is descriptive data and cannot be converted to command evidence.
/// ```
/// use worth_query_execution::facade::application_discovery::WorthQueryWorkflowObservation;
/// fn inspect(view:&WorthQueryWorkflowObservation) { let _ = (view.state(),view.recorded_paths(),view.selected_commit()); }
/// ```
/// ```compile_fail,E0599
/// use worth_query_execution::facade::application_discovery::WorthQueryWorkflowObservation;
/// fn cannot_advance(view:WorthQueryWorkflowObservation) { view.advance(); }
/// ```
pub struct WorthQueryWorkflowObservation {
    pub(super) basis: WorthQueryWorkflowReadBasis,
    pub(super) state: WorthQueryWorkflowObservedState,
    pub(super) history: ReadOnlyWorkflowHistory,
    pub(super) audit: WorthQueryWorkflowAudit,
}
impl WorthQueryWorkflowObservation {
    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.basis.selected_commit()
    }
    pub fn state(&self) -> WorthQueryWorkflowObservedState {
        self.state
    }
    pub fn transition_count(&self) -> usize {
        self.history.transition_count()
    }
    pub fn recorded_paths(&self) -> &[String] {
        self.history.recorded_paths()
    }
    pub fn audit(&self) -> &WorthQueryWorkflowAudit {
        &self.audit
    }
}

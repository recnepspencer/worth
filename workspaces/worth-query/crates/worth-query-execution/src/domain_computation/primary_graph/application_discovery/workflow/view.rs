//! Governed data values. No view or historical actor grants command authority.
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::ReadOnlyWorkflowHistory;
use crate::domain_computation::primary_graph::application_attempt::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::ReadObservationRuntimeExt;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::application_discovery::{
    WorthQueryWorkflowAudit, WorthQueryWorkflowObservedState, WorthQueryWorkflowTransitionActors,
};
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
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_discovery::workflow::WorthQueryWorkflowReadBasis;
pub(super) trait WorthQueryWorkflowReadBasisRuntimeExt {
    fn retained<Schema: worth_query_installation::facade::ApplicationSchema>(
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Self;
}
impl WorthQueryWorkflowReadBasisRuntimeExt for WorthQueryWorkflowReadBasis {
    fn retained<Schema: worth_query_installation::facade::ApplicationSchema>(
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Self {
        // Retain only the native read lease for the result's observer lifetime.
        // The selected operation's publication source and fresh admission stay private.
        Self(WorthQueryApplicationReadObservation::from_product(
            selected.application().inner(),
            selected.product().read_lease(),
        ))
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
    pub(super) target: super::target::WorthQueryWorkflowReadTarget,
    pub(super) audit: WorthQueryWorkflowAudit,
}
impl WorthQueryWorkflowObservation {
    /// Opaque exact-data locator; each use independently admits current disclosure.
    pub fn target(&self) -> &super::target::WorthQueryWorkflowReadTarget {
        &self.target
    }
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

impl WorthQueryWorkflowInstanceRead {
    /// Missing is not a locator. Historical locators remain descriptive and freshly governed.
    pub fn target(&self) -> Option<&super::target::WorthQueryWorkflowReadTarget> {
        match self {
            Self::Missing { .. } => None,
            Self::Live { observation, .. } | Self::Historical { observation, .. } => {
                Some(observation.target())
            }
        }
    }
}

use worth_relational::facade::identity::EntityId;

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::primary_graph::workflow::instance::{
    SelectedWorkflowTransition, SelectedWorkflowTransitionKind,
};

/// The selected node's control outcome, observed before actor admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequiredWorkflowActorNodeKind {
    Operation,
    Assessment,
    Condition,
    Approval,
    EvidenceJoin,
    NavigationBack,
    Terminal,
}

/// A current, live workflow head observed after the advancing actor was denied.
/// This is descriptive readiness, not permission or a prepared transition.
#[derive(Clone, Debug)]
pub struct RequiredWorkflowActor {
    instance: EntityId,
    node_path: String,
    transition_identity: String,
    occurrence: u64,
    node_kind: RequiredWorkflowActorNodeKind,
    denial: WorthQueryOperationAuthorizationDenial,
}

impl RequiredWorkflowActor {
    pub(in crate::domain_computation::primary_graph) fn new(
        instance: EntityId,
        denial: WorthQueryOperationAuthorizationDenial,
        selected: &SelectedWorkflowTransition,
    ) -> Self {
        let node_kind = match selected.kind() {
            SelectedWorkflowTransitionKind::Operation(_) => {
                RequiredWorkflowActorNodeKind::Operation
            }
            SelectedWorkflowTransitionKind::Assessment(_) => {
                RequiredWorkflowActorNodeKind::Assessment
            }
            SelectedWorkflowTransitionKind::Condition(_) => {
                RequiredWorkflowActorNodeKind::Condition
            }
            SelectedWorkflowTransitionKind::Approval(_) => RequiredWorkflowActorNodeKind::Approval,
            SelectedWorkflowTransitionKind::EvidenceJoin(_) => {
                RequiredWorkflowActorNodeKind::EvidenceJoin
            }
            SelectedWorkflowTransitionKind::NavigationBack => {
                RequiredWorkflowActorNodeKind::NavigationBack
            }
            SelectedWorkflowTransitionKind::Terminal => RequiredWorkflowActorNodeKind::Terminal,
        };
        Self {
            instance,
            node_path: selected.node_path().to_owned(),
            transition_identity: selected.identity().to_owned(),
            occurrence: selected.occurrence(),
            node_kind,
            denial,
        }
    }

    pub const fn instance(&self) -> EntityId {
        self.instance
    }

    /// Disclosure is limited to the live selected node's path, occurrence, kind,
    /// and opaque identity; this observation does not carry its inputs or authority.
    pub fn node_path(&self) -> &str {
        &self.node_path
    }

    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }

    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub const fn node_kind(&self) -> RequiredWorkflowActorNodeKind {
        self.node_kind
    }

    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }
}

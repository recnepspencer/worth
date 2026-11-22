//! Crate-boundary internal ports for workflow history pages and recovered
//! program output sources. Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;

use worth_execution::ExecutionArray;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::BoundedRelatedEntityPrefixPage;
use worth_runtime_world::facade::ProductBranchObservation;

use crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange;
use crate::domain_computation::primary_graph::application_discovery::workflow::{
    WorthQueryWorkflowHistoryEntry, WorthQueryWorkflowHistoryPage, WorthQueryWorkflowReadBasis,
    WorthQueryWorkflowRecordedActor,
};
use crate::domain_computation::primary_graph::application_installation::output_source_recovery::WorthQueryRecoveredProgramOutputSource;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryCommittedProductPublication,
};

pub struct WorkflowHistoryPageParts {
    pub basis: WorthQueryWorkflowReadBasis,
    pub instance: EntityId,
    pub subject: EntityId,
    pub scope: EntityId,
    pub support: [u8; 32],
    pub width: usize,
    pub rows: ExecutionArray<WorthQueryWorkflowHistoryEntry>,
    pub native: BoundedRelatedEntityPrefixPage,
}

pub trait WorkflowHistoryPageInternalPort: Sized {
    fn from_parts(parts: WorkflowHistoryPageParts) -> Self;
    fn basis_ref(&self) -> &WorthQueryWorkflowReadBasis;
    fn instance(&self) -> EntityId;
    fn subject(&self) -> EntityId;
    fn scope(&self) -> EntityId;
    fn support(&self) -> &[u8; 32];
    fn width(&self) -> usize;
    fn native(&self) -> &BoundedRelatedEntityPrefixPage;
}

impl WorkflowHistoryPageInternalPort for WorthQueryWorkflowHistoryPage {
    fn from_parts(parts: WorkflowHistoryPageParts) -> Self {
        Self {
            basis: parts.basis,
            instance: parts.instance,
            subject: parts.subject,
            scope: parts.scope,
            support: parts.support,
            width: parts.width,
            rows: parts.rows,
            native: parts.native,
        }
    }

    fn basis_ref(&self) -> &WorthQueryWorkflowReadBasis {
        &self.basis
    }

    fn instance(&self) -> EntityId {
        self.instance
    }

    fn subject(&self) -> EntityId {
        self.subject
    }

    fn scope(&self) -> EntityId {
        self.scope
    }

    fn support(&self) -> &[u8; 32] {
        &self.support
    }

    fn width(&self) -> usize {
        self.width
    }

    fn native(&self) -> &BoundedRelatedEntityPrefixPage {
        &self.native
    }
}

pub trait RecoveredProgramOutputSourceInternalPort {
    fn publication_ref(&self) -> &WorthQueryCommittedProductPublication;
    fn change_ref(&self) -> Option<&Arc<WorthQueryPerformedRelationalProductChange>>;
    fn observation_ref(&self) -> Option<&ProductBranchObservation>;
}

impl RecoveredProgramOutputSourceInternalPort for WorthQueryRecoveredProgramOutputSource {
    fn publication_ref(&self) -> &WorthQueryCommittedProductPublication {
        &self.publication
    }

    fn change_ref(&self) -> Option<&Arc<WorthQueryPerformedRelationalProductChange>> {
        self.change.as_ref()
    }

    fn observation_ref(&self) -> Option<&ProductBranchObservation> {
        self.observation.as_ref()
    }
}

pub trait WorkflowRecordedActorInternalPort: Sized {
    fn observed(entity: EntityId) -> Self;
}

impl WorkflowRecordedActorInternalPort for WorthQueryWorkflowRecordedActor {
    fn observed(entity: EntityId) -> Self {
        Self::observed(entity)
    }
}

pub trait WorkflowReadBasisInternalPort: Sized {
    fn from_observation(observation: Arc<WorthQueryApplicationReadObservation>) -> Self;
    fn observation(&self) -> &WorthQueryApplicationReadObservation;
}

impl WorkflowReadBasisInternalPort for WorthQueryWorkflowReadBasis {
    fn from_observation(observation: Arc<WorthQueryApplicationReadObservation>) -> Self {
        Self::from_observation(observation)
    }

    fn observation(&self) -> &WorthQueryApplicationReadObservation {
        Self::observation(self)
    }
}

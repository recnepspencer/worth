use std::sync::{Arc, Mutex};

use worth_relational::facade::change_source::RelationalRuntimeHandle;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::runtime::RelationalRuntime;

mod branch_basis;
mod branch_head_bindings;
mod branch_heads;
mod canonical_subscription;
mod committed_patches;
mod continuity_lineage;
mod observation_bindings;
mod retained_entity_projection;
mod retained_snapshot;
mod selected_commit_resolution;
mod snapshot_reads;
mod source_profile;

pub use branch_head_bindings::{
    RelationalBridgeBranchHeadLease, RelationalBridgeBranchHeadReleaseReceipt,
};
pub use canonical_subscription::{
    PendingRelationalBridgeCanonicalSubscription, RelationalBridgeCanonicalSubscription,
};
pub use observation_bindings::{
    RelationalBridgeObservationLease, RelationalBridgeObservationReleaseReceipt,
};
pub(in crate::relational_source) use observation_bindings::{
    RelationalBridgeSelectedCommitObservation, RelationalBridgeSelectedObservation,
};
pub use retained_snapshot::RelationalBridgeRetainedSnapshot;

#[derive(Debug, Clone)]
pub struct RuntimeBridgeRelationalSource {
    runtime: RelationalRuntimeHandle,
    observation_bindings: Arc<observation_bindings::RelationalBridgeObservationBindings>,
    branch_head_bindings: Arc<branch_head_bindings::RelationalBridgeBranchHeadBindings>,
    graph_role: Arc<str>,
    partition: Option<RelationalBridgePartitionBinding>,
    selection_work: Arc<selected_commit_resolution::SourceSelectionWork>,
}

#[derive(Debug, Clone)]
struct RelationalBridgePartitionBinding {
    relational: PartitionId,
    truth: worth_foundational::facade::TruthPartitionRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalBridgeSourceConfigurationError {
    InvalidGraphRole,
}

impl RuntimeBridgeRelationalSource {
    pub fn for_graph_role(
        runtime: Arc<RelationalRuntime>,
        graph_role: impl Into<Arc<str>>,
    ) -> Result<Self, RelationalBridgeSourceConfigurationError> {
        let graph_role = graph_role.into();
        validate_graph_role(&graph_role)?;
        Ok(Self {
            runtime: RelationalRuntimeHandle::immutable(runtime),
            observation_bindings: observation_bindings::RelationalBridgeObservationBindings::new(),
            branch_head_bindings: branch_head_bindings::RelationalBridgeBranchHeadBindings::new(),
            graph_role,
            partition: None,
            selection_work: Arc::default(),
        })
    }

    pub fn for_shared_graph_role(
        runtime: Arc<Mutex<RelationalRuntime>>,
        graph_role: impl Into<Arc<str>>,
    ) -> Result<Self, RelationalBridgeSourceConfigurationError> {
        let graph_role = graph_role.into();
        validate_graph_role(&graph_role)?;
        Ok(Self {
            runtime: RelationalRuntimeHandle::shared(runtime),
            observation_bindings: observation_bindings::RelationalBridgeObservationBindings::new(),
            branch_head_bindings: branch_head_bindings::RelationalBridgeBranchHeadBindings::new(),
            graph_role,
            partition: None,
            selection_work: Arc::default(),
        })
    }

    pub fn for_shared_graph_partition(
        runtime: Arc<Mutex<RelationalRuntime>>,
        graph_role: impl Into<Arc<str>>,
        relational_partition: PartitionId,
        truth_partition: worth_foundational::facade::TruthPartitionRole,
    ) -> Result<Self, RelationalBridgeSourceConfigurationError> {
        let mut source = Self::for_shared_graph_role(runtime, graph_role)?;
        source.partition = Some(RelationalBridgePartitionBinding {
            relational: relational_partition,
            truth: truth_partition,
        });
        Ok(source)
    }

    pub fn for_graph_partition(
        runtime: Arc<RelationalRuntime>,
        graph_role: impl Into<Arc<str>>,
        relational_partition: PartitionId,
        truth_partition: worth_foundational::facade::TruthPartitionRole,
    ) -> Result<Self, RelationalBridgeSourceConfigurationError> {
        let mut source = Self::for_graph_role(runtime, graph_role)?;
        source.partition = Some(RelationalBridgePartitionBinding {
            relational: relational_partition,
            truth: truth_partition,
        });
        Ok(source)
    }

    fn admits_relational_partition(&self, partition_id: u32) -> bool {
        self.partition
            .as_ref()
            .is_none_or(|partition| partition.relational.as_u32() == partition_id)
    }
}

fn validate_graph_role(
    graph_role: &Arc<str>,
) -> Result<(), RelationalBridgeSourceConfigurationError> {
    if graph_role.trim().is_empty() || graph_role.trim() != graph_role.as_ref() {
        Err(RelationalBridgeSourceConfigurationError::InvalidGraphRole)
    } else {
        Ok(())
    }
}

use std::cell::RefCell;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::visibility_read_view::StrategyVisibilityReadView;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
};
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::materialization::read_records::VisibilityProjectionView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StrategyExecutionSummary {
    pub projected_entity_record_reads: usize,
    pub projected_relation_record_reads: usize,
    pub projected_partition_reads: usize,
}

#[derive(Debug)]
pub struct StrategyObservationContext<'runtime> {
    snapshot: &'runtime SnapshotHandle,
    read_contract: &'runtime crate::commit_strategies::data::StrategyReadContract,
    schema_registry: &'runtime RelationalSchemaRegistry,
    aspect_plans: &'runtime AspectContractPlanCatalog,
    metrics: RefCell<StrategyObservationMetrics>,
    projection: VisibilityProjectionView,
}

impl<'runtime> StrategyObservationContext<'runtime> {
    pub(crate) fn new(
        _runtime: &'runtime crate::runtime::RelationalRuntime,
        snapshot: &'runtime SnapshotHandle,
        read_contract: &'runtime crate::commit_strategies::data::StrategyReadContract,
        schema_registry: &'runtime RelationalSchemaRegistry,
        aspect_plans: &'runtime AspectContractPlanCatalog,
        visibility: VisibilityProjectionView,
    ) -> Self {
        Self {
            snapshot,
            read_contract,
            schema_registry,
            aspect_plans,
            metrics: RefCell::new(StrategyObservationMetrics::default()),
            projection: visibility,
        }
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        self.snapshot
    }

    pub fn version_id(&self) -> VersionId {
        self.projection.version_id()
    }

    pub fn read_contract(&self) -> &crate::commit_strategies::data::StrategyReadContract {
        self.read_contract
    }

    pub fn schema_registry(&self) -> &RelationalSchemaRegistry {
        self.schema_registry
    }

    pub fn entity_aspect_plan(&self, kind_id: KindId) -> Option<&LoweredAspectContractPlan> {
        self.aspect_plans.entity_plans.get(&kind_id)
    }

    pub fn visibility(&self) -> StrategyVisibilityReadView<'_, 'runtime> {
        StrategyVisibilityReadView::new(&self.projection, self.read_contract, &self.metrics)
    }

    pub(crate) fn measured_summary(&self) -> StrategyExecutionSummary {
        self.metrics.borrow().summary()
    }
}

#[derive(Debug, Default)]
pub(super) struct StrategyObservationMetrics {
    pub(super) projected_entity_record_reads: usize,
    pub(super) projected_relation_record_reads: usize,
    pub(super) partitions_touched: BTreeSet<PartitionId>,
    pub(super) first_partition: Option<PartitionId>,
}

impl StrategyObservationMetrics {
    fn summary(&self) -> StrategyExecutionSummary {
        StrategyExecutionSummary {
            projected_entity_record_reads: self.projected_entity_record_reads,
            projected_relation_record_reads: self.projected_relation_record_reads,
            projected_partition_reads: self.partitions_touched.len(),
        }
    }
}

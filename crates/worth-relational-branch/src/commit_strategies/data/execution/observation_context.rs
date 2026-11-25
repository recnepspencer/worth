use std::cell::RefCell;
use std::collections::BTreeSet;

use super::visibility_read_view::StrategyVisibilityReadView;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::schema::data::{
    AspectContractPlanCatalog, LoweredAspectContractPlan, RelationalSchemaRegistry,
};
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;

use super::execution_summary::StrategyExecutionSummary;

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

/// Operations used by the Relational owners.
pub trait StrategyObservationContextOwnerPort<'runtime>: Sized {
    fn new(
        snapshot: &'runtime SnapshotHandle,
        read_contract: &'runtime crate::commit_strategies::data::StrategyReadContract,
        schema_registry: &'runtime RelationalSchemaRegistry,
        aspect_plans: &'runtime AspectContractPlanCatalog,
        visibility: VisibilityProjectionView,
    ) -> Self;
    fn measured_summary(&self) -> StrategyExecutionSummary;
}

impl<'runtime> StrategyObservationContextOwnerPort<'runtime>
    for StrategyObservationContext<'runtime>
{
    fn new(
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

    fn measured_summary(&self) -> StrategyExecutionSummary {
        self.metrics.borrow().summary()
    }
}

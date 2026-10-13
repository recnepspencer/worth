use std::cell::RefCell;

use super::observation_context::StrategyObservationMetrics;
use crate::commit_strategies::data::{
    StrategyEntityAspectReadRecord, StrategyExecutorFailure, StrategyExecutorFailureClass,
    StrategyPacketContract, StrategyReadContract, StrategyReadLocalityClass,
    StrategyReadScopeClass,
};
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::visibility::materialization::read_records::{
    EntityRecordProjection, ProjectionAspectScope, RelationRecordProjection,
    VisibilityProjectionView,
};

#[derive(Debug, Clone, Copy)]
pub struct StrategyVisibilityReadView<'observation, 'runtime> {
    projection: &'observation VisibilityProjectionView,
    read_contract: &'runtime StrategyReadContract,
    metrics: &'observation RefCell<StrategyObservationMetrics>,
}

impl<'observation, 'runtime> StrategyVisibilityReadView<'observation, 'runtime> {
    pub(super) const fn new(
        projection: &'observation VisibilityProjectionView,
        read_contract: &'runtime StrategyReadContract,
        metrics: &'observation RefCell<StrategyObservationMetrics>,
    ) -> Self {
        Self {
            projection,
            read_contract,
            metrics,
        }
    }

    pub fn version_id(&self) -> VersionId {
        self.projection.version_id()
    }

    pub fn entities<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, StrategyExecutorFailure> {
        self.admit_kind_scan(None, "cross-partition entity kind scan")?;
        let values = self.safe_projection("cross-partition entity kind scan", || {
            self.projection.entities::<T>(execution)
        })?;
        self.record_entity_reads(None, values.len());
        Ok(values)
    }

    pub fn entities_in<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
    ) -> Result<Vec<T>, StrategyExecutorFailure> {
        self.admit_kind_scan(Some(partition_id), "partition-scoped entity kind scan")?;
        let values = self.safe_projection("partition-scoped entity kind scan", || {
            self.projection.entities_in::<T>(execution, partition_id)
        })?;
        self.record_entity_reads(Some(partition_id), values.len());
        Ok(values)
    }

    pub fn entity<T: EntityRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<Option<T>, StrategyExecutorFailure> {
        self.admit_target_lookup(entity_id.partition_id, "entity lookup")?;
        let value = self.safe_projection("entity lookup", || {
            self.projection.entity::<T>(execution, entity_id)
        })?;
        self.record_entity_reads(Some(entity_id.partition_id), usize::from(value.is_some()));
        Ok(value)
    }

    pub fn relations<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<T>, StrategyExecutorFailure> {
        self.admit_kind_scan(None, "cross-partition relation kind scan")?;
        let values = self.safe_projection("cross-partition relation kind scan", || {
            self.projection.relations::<T>(execution)
        })?;
        self.record_relation_reads(None, values.len());
        Ok(values)
    }

    pub fn relations_in<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
    ) -> Result<Vec<T>, StrategyExecutorFailure> {
        self.admit_kind_scan(Some(partition_id), "partition-scoped relation kind scan")?;
        let values = self.safe_projection("partition-scoped relation kind scan", || {
            self.projection.relations_in::<T>(execution, partition_id)
        })?;
        self.record_relation_reads(Some(partition_id), values.len());
        Ok(values)
    }

    pub fn relation<T: RelationRecordProjection>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation_id: RelationId,
    ) -> Result<Option<T>, StrategyExecutorFailure> {
        self.admit_target_lookup(relation_id.partition_id, "relation lookup")?;
        let value = self.safe_projection("relation lookup", || {
            self.projection.relation::<T>(execution, relation_id)
        })?;
        self.record_relation_reads(Some(relation_id.partition_id), usize::from(value.is_some()));
        Ok(value)
    }

    pub fn entity_whole_aspects(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        aspect_keys: impl IntoIterator<Item = worth_foundational::facade::AspectKey>,
    ) -> Result<Option<StrategyEntityAspectReadRecord>, StrategyExecutorFailure> {
        self.admit_target_lookup(entity_id.partition_id, "entity whole-aspect lookup")?;
        let aspect_keys = canonical_aspect_key_set(aspect_keys);
        let projection_scope = ProjectionAspectScope::whole_aspects(aspect_keys.clone());
        let value = self.safe_projection("entity whole-aspect lookup", || {
            self.projection.entity_record_with_projection_scope(
                execution,
                entity_id,
                projection_scope,
                |record| {
                    Some(StrategyEntityAspectReadRecord::from_projection(
                        record,
                        &aspect_keys,
                    ))
                },
            )
        })?;
        self.record_entity_reads(Some(entity_id.partition_id), usize::from(value.is_some()));
        Ok(value)
    }

    fn admit_target_lookup(
        &self,
        partition_id: PartitionId,
        operation: &'static str,
    ) -> Result<(), StrategyExecutorFailure> {
        self.reject_if_packet_plan_only(operation)?;
        self.enforce_locality(partition_id, operation)
    }

    fn admit_kind_scan(
        &self,
        partition_id: Option<PartitionId>,
        operation: &'static str,
    ) -> Result<(), StrategyExecutorFailure> {
        self.reject_if_packet_plan_only(operation)?;
        match self.read_contract.scope_class {
            StrategyReadScopeClass::ExplicitTargetsOnly => {
                return Err(StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::ReadContractViolation,
                    format!("{operation} is forbidden by ExplicitTargetsOnly strategy read scope"),
                ));
            }
            StrategyReadScopeClass::PartitionBoundedScan if partition_id.is_none() => {
                return Err(StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::ReadContractViolation,
                    format!("{operation} is forbidden by PartitionBoundedScan strategy read scope"),
                ));
            }
            StrategyReadScopeClass::BoundedNeighborhood => {
                return Err(StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::ReadContractViolation,
                    format!(
                        "{operation} is forbidden because bounded-neighborhood traversal is not exposed yet"
                    ),
                ));
            }
            StrategyReadScopeClass::KindBoundedScan
            | StrategyReadScopeClass::PartitionBoundedScan => {}
        }
        if let Some(partition_id) = partition_id {
            self.enforce_locality(partition_id, operation)?;
        } else if matches!(
            self.read_contract.locality_class,
            StrategyReadLocalityClass::SinglePartition
        ) {
            return Err(StrategyExecutorFailure::new(
                StrategyExecutorFailureClass::ReadContractViolation,
                format!("{operation} is forbidden by SinglePartition strategy locality"),
            ));
        }
        Ok(())
    }

    fn reject_if_packet_plan_only(
        &self,
        operation: &'static str,
    ) -> Result<(), StrategyExecutorFailure> {
        if matches!(
            self.read_contract.packet_contract,
            StrategyPacketContract::PlannedPacketOnly
        ) {
            return Err(StrategyExecutorFailure::new(
                StrategyExecutorFailureClass::ReadContractViolation,
                format!(
                    "{operation} requires planned packet reads, but packet-planned strategy execution is not implemented yet"
                ),
            ));
        }
        Ok(())
    }

    fn enforce_locality(
        &self,
        partition_id: PartitionId,
        operation: &'static str,
    ) -> Result<(), StrategyExecutorFailure> {
        let mut metrics = self.metrics.borrow_mut();
        if matches!(
            self.read_contract.locality_class,
            StrategyReadLocalityClass::SinglePartition
        ) {
            match metrics.first_partition {
                Some(first_partition) if first_partition != partition_id => {
                    return Err(StrategyExecutorFailure::new(
                        StrategyExecutorFailureClass::ReadContractViolation,
                        format!(
                            "{operation} crossed from partition {} into partition {} under SinglePartition locality",
                            first_partition.0, partition_id.0
                        ),
                    ));
                }
                None => metrics.first_partition = Some(partition_id),
                Some(_) => {}
            }
        }
        Ok(())
    }

    fn record_entity_reads(&self, partition_id: Option<PartitionId>, count: usize) {
        if count == 0 {
            return;
        }
        let mut metrics = self.metrics.borrow_mut();
        metrics.projected_entity_record_reads += count;
        if let Some(partition_id) = partition_id {
            metrics.partitions_touched.insert(partition_id);
        }
    }

    fn record_relation_reads(&self, partition_id: Option<PartitionId>, count: usize) {
        if count == 0 {
            return;
        }
        let mut metrics = self.metrics.borrow_mut();
        metrics.projected_relation_record_reads += count;
        if let Some(partition_id) = partition_id {
            metrics.partitions_touched.insert(partition_id);
        }
    }

    fn projection_failure(
        &self,
        operation: &'static str,
        unwind_value: Box<dyn std::any::Any + Send>,
    ) -> StrategyExecutorFailure {
        let detail = if let Some(message) = unwind_value.downcast_ref::<&'static str>() {
            (*message).to_string()
        } else if let Some(message) = unwind_value.downcast_ref::<String>() {
            message.clone()
        } else {
            format!("{operation} violated the projection contract")
        };
        StrategyExecutorFailure::new(
            StrategyExecutorFailureClass::ProjectionContractViolation,
            detail,
        )
    }

    fn safe_projection<T>(
        &self,
        operation: &'static str,
        run: impl FnOnce() -> Result<T, crate::execution::RelationalExecutionDenialCause>,
    ) -> Result<T, StrategyExecutorFailure> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(run))
            .map_err(|unwind_value| self.projection_failure(operation, unwind_value))?
            .map_err(|cause| {
                StrategyExecutorFailure::new(
                    StrategyExecutorFailureClass::Execution(cause),
                    operation,
                )
            })
    }
}

fn canonical_aspect_key_set(
    aspect_keys: impl IntoIterator<Item = worth_foundational::facade::AspectKey>,
) -> Vec<worth_foundational::facade::AspectKey> {
    let mut aspect_keys = aspect_keys.into_iter().collect::<Vec<_>>();
    aspect_keys.sort();
    aspect_keys.dedup();
    aspect_keys
}

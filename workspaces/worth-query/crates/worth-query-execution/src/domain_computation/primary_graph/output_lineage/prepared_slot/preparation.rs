//! Selected successor lineage address preparation before World publication.

use super::{
    arc_bytes, denial, tree_insert_bytes, tree_work, CancelledLineageSlot,
    PreparedOutputLineageSlot,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, ProductCoordinate, RecordedGeneration, RecordedOutput,
    RecordedSettlementIdentity, SemanticSource, WorthQueryApplicationOutputLineage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use std::{
    any::TypeId,
    mem::size_of,
    sync::{Arc, Mutex, OnceLock},
};
use worth_runtime_world::facade::PlannedProductReferenceSuccessor;

pub(in crate::domain_computation::primary_graph) fn prepare(
    owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    scope: &WorthQueryOperationScopeBinding,
    output_binding: TypeId,
    partition: Option<[u8; 32]>,
    planned: &PlannedProductReferenceSuccessor,
    admission: &mut InvalidationEditAdmission,
) -> Result<PreparedOutputLineageSlot, WorthQueryOutputDemandDenial> {
    use WorthQueryOutputDemandDenialKind as Kind;

    // The descriptive source clones one fixed schema identity and initializes
    // three scalar fields before it can probe the owner indexes.
    admission
        .charge_external_work(4 + std::mem::size_of::<Option<crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources>>() as u64)
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let source = SemanticSource {
        runtime_authority: scope.runtime_authority(),
        schema: scope.binding_identity().clone(),
        scope: scope.scope(),
        output_binding,
    };
    let coordinate = ProductCoordinate {
        occurrence: planned.occurrence(),
        generation: planned.generation().get(),
    };
    let mut lineage = owner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lineage.drain_cancelled_slots(admission)?;
    // History no retained reader selects is freed before this address extends it.
    lineage.retire_unselected_generations(&source, coordinate.occurrence, admission)?;
    // Charge each selected lookup before performing it. No accumulated
    // history is traversed to reserve a single product address.
    admission
        .charge_external_work(
            tree_work::<SemanticSource>(lineage.by_source.len())
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let occurrences = lineage.by_source.get(&source);
    let occurrence_count = occurrences.map_or(0, |value| value.len());
    admission
        .charge_external_work(
            tree_work::<worth_runtime_world::facade::ProductBranchIncarnation>(occurrence_count)
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let history = occurrences.and_then(|value| value.get(&coordinate.occurrence));
    admission
        .charge_external_work(
            tree_work::<u64>(history.map_or(0, |value| value.len()))
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    if history.is_some_and(|value| value.contains_key(&coordinate.generation)) {
        return Err(denial(Kind::SchedulingDeferred));
    }
    // The selected entry paths are walked again when the vacant history is
    // inserted; that work is separate from the lookups above.
    admission
        .charge_external_work(
            tree_work::<SemanticSource>(lineage.by_source.len())
                .and_then(|work| {
                    work.checked_add(tree_work::<
                        worth_runtime_world::facade::ProductBranchIncarnation,
                    >(occurrence_count)?)
                })
                .and_then(|work| {
                    work.checked_add(tree_work::<u64>(history.map_or(0, |value| value.len()))?)
                })
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let mut bytes =
        tree_insert_bytes::<u64, RecordedGeneration>(history.map_or(0, |value| value.len()))
            .and_then(|value| value.checked_add(size_of::<Arc<OnceLock<RecordedOutput>>>() as u64))
            .and_then(|value| value.checked_add(arc_bytes::<OnceLock<RecordedOutput>>()?))
            .and_then(|value| value.checked_add(size_of::<CancelledLineageSlot>() as u64))
            .and_then(|value| value.checked_add(arc_bytes::<RecordedSettlementIdentity>()?))
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    if occurrences.is_none() {
        bytes = bytes
            .checked_add(
                tree_insert_bytes::<
                    SemanticSource,
                    std::collections::BTreeMap<
                        worth_runtime_world::facade::ProductBranchIncarnation,
                        std::collections::BTreeMap<u64, RecordedGeneration>,
                    >,
                >(lineage.by_source.len())
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
            )
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    }
    if history.is_none() {
        bytes = bytes
            .checked_add(
                tree_insert_bytes::<
                    worth_runtime_world::facade::ProductBranchIncarnation,
                    std::collections::BTreeMap<u64, RecordedGeneration>,
                >(occurrence_count)
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
            )
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    }
    bytes = bytes
        .checked_add(
            lineage
                .partition_index
                .vacancy_preparation_bytes(&source, coordinate, partition, admission)?,
        )
        .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    admission
        .charge_external_work(
            tree_work::<worth_runtime_world::facade::ProductBranchIncarnation>(
                lineage.live_occurrences.len(),
            )
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    if !lineage.live_occurrences.contains(&coordinate.occurrence) {
        admission
            .charge_external_work(
                tree_work::<worth_runtime_world::facade::ProductBranchIncarnation>(
                    lineage.live_occurrences.len(),
                )
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        bytes = bytes
            .checked_add(
                tree_insert_bytes::<worth_runtime_world::facade::ProductBranchIncarnation, ()>(
                    lineage.live_occurrences.len(),
                )
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
            )
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    }
    // Every retained row pays an unconditional source and occurrence path.
    // Its token remains sound if an earlier row of the same source retires.
    let retained_bytes = bytes
        .checked_add(
            tree_insert_bytes::<
                SemanticSource,
                std::collections::BTreeMap<
                    worth_runtime_world::facade::ProductBranchIncarnation,
                    std::collections::BTreeMap<u64, RecordedGeneration>,
                >,
            >(lineage.by_source.len())
            .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
        )
        .and_then(|bytes| {
            bytes.checked_add(tree_insert_bytes::<
                worth_runtime_world::facade::ProductBranchIncarnation,
                std::collections::BTreeMap<u64, RecordedGeneration>,
            >(occurrence_count)?)
        })
        .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    let retained_bytes = retained_bytes
        .checked_add(
            lineage
                .partition_index
                .retained_path_guard_bytes(&source, coordinate, partition)
                .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?,
        )
        .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    let retained_capacity = lineage.retention.reserve(retained_bytes)?;
    admission
        .admit_read_scratch(bytes)
        .map_err(|_| denial(Kind::RetentionBudgetExceeded))?;
    // The settlement identity and both owner indexes each clone one fixed
    // source key. Inline key widths are part of retained capacity.
    admission
        .charge_external_work(3)
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(1)
        .map_err(|_| denial(Kind::RetentionBudgetExceeded))?;
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let record_cell = Arc::new(OnceLock::new());
    records.push(Arc::clone(&record_cell));
    let cancellation = Box::new(CancelledLineageSlot {
        identity: Arc::clone(&identity),
        partition,
        retained_capacity: None,
        next: None,
    });
    lineage
        .by_source
        .entry(source.clone())
        .or_default()
        .entry(coordinate.occurrence)
        .or_default()
        .insert(coordinate.generation, records);
    let partition_cell =
        lineage
            .partition_index
            .insert_vacancy(source.clone(), coordinate, partition);
    lineage.live_occurrences.insert(coordinate.occurrence);
    drop(lineage);
    Ok(PreparedOutputLineageSlot {
        owner: Arc::clone(owner),
        source,
        coordinate,
        partition,
        identity,
        record_cell,
        partition_cell,
        retained_capacity: Some(retained_capacity),
        cancellation: Some(cancellation),
        completed_handler_facts: None,
        completed_decision_reuse: None,
        prepared_input_reuse_key: None,
        native_output_witness: None,
        actual_resources: None,
        native_prior_checkpoint: None,
        filled: false,
    })
}

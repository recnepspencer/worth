//! Selected successor lineage address preparation before World publication.

use super::denial;
use super::CancelledLineageSlot;
use super::PreparedComputationCustody;
use super::PreparedOutputLineageSlot;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use worth_runtime_world::facade::PlannedProductReferenceSuccessor;

pub(in crate::domain_computation::primary_graph) fn prepare(
    owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    scope: &WorthQueryOperationScopeBinding,
    output_binding: TypeId,
    partition: Option<[u8; 32]>,
    planned: &PlannedProductReferenceSuccessor,
) -> Result<PreparedOutputLineageSlot, WorthQueryOutputDemandDenial> {
    use WorthQueryOutputDemandDenialKind as Kind;

    // The descriptive source clones one fixed schema identity and initializes
    // three scalar fields before it can probe the owner indexes.

    let mut lineage = owner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let source = SemanticSource {
        runtime_authority: scope.runtime_authority(),
        schema: scope.binding_identity().clone(),
        scope: scope.scope(),
        output_binding: lineage
            .binding_identity(output_binding)
            .expect("admitted binding is installed"),
    };
    let coordinate = ProductCoordinate {
        occurrence: planned.occurrence(),
        generation: planned.generation().get(),
    };
    lineage.drain_cancelled_slots()?;
    // History no retained reader selects is freed before this address extends it.
    // Charge each selected lookup before performing it. No accumulated
    // history is traversed to reserve a single product address.

    let occurrences = lineage.by_source.get(&source);

    let history = occurrences.and_then(|value| value.get(&coordinate.occurrence));

    if history.is_some_and(|value| value.contains_key(&coordinate.generation)) {
        return Err(denial(Kind::SchedulingDeferred));
    }
    // The selected entry paths are walked again when the vacant history is
    // inserted; that work is separate from the lookups above.

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
        required_publication: None,
        owner: Arc::clone(owner),
        output_binding_type: output_binding,
        source,
        coordinate,
        partition,
        identity,
        record_cell,
        partition_cell,
        cancellation: Some(cancellation),
        completed_handler_facts: None,
        completed_decision_reuse: None,
        prepared_input_reuse_key: None,
        native_output_witness: None,

        native_prior_checkpoint: None,
        computation: PreparedComputationCustody::Unassigned,
        prior_computation: None,
        filled: false,
    })
}

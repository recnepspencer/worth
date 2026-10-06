//! Prospective stable lineage addresses held by the existing occurrence lane.

mod performed_sequence;
mod source_registration;
mod stable_record;
pub(in crate::domain_computation::primary_graph::output_lineage) use performed_sequence::performed_fact_sequence;
pub(in crate::domain_computation::primary_graph) use source_registration::StableEqualityConsequence;

use std::sync::{Arc, Mutex, OnceLock};

use worth_runtime_world::facade::ProductBranchObservation;

use super::{RetainedInputCutoffCandidate, VerifiedInputCutoff};
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::InvalidationEditAdmission,
        prepared_slot::{arc_bytes, denial, tree_work},
        retained_capacity::RetainedLineageCapacity,
        ProductCoordinate, RecordedOutput, RecordedSettlementIdentity, SemanticSource,
        WorthQueryApplicationOutputLineage,
    },
    provider::WorthQueryApplicationBranchCommitCoordination,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
};

/// This reserves preparation custody and a prospective address, not a published
/// settlement. Borrowing the coordination proof keeps the exact occurrence
/// lane held while the registry prepares its own prerequisite vacancy.
pub(in crate::domain_computation::primary_graph) struct PreparedStableLineageAddress<
    'lane,
    'selected,
> {
    pub(super) owner: Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    pub(super) source: SemanticSource,
    pub(super) coordinate: ProductCoordinate,
    pub(super) partition: [u8; 32],
    pub(super) identity: Arc<RecordedSettlementIdentity>,
    pub(super) record_cell: Arc<OnceLock<RecordedOutput>>,
    pub(super) expected_record_count: usize,
    pub(super) verified: VerifiedInputCutoff<'selected>,
    pub(super) retained_capacity: Option<RetainedLineageCapacity>,
    pub(super) _coordination: &'lane WorthQueryApplicationBranchCommitCoordination<'lane>,
}

/// A fully materialized, private alias row awaiting the exact-current cutover.
pub(in crate::domain_computation::primary_graph) struct PreparedStableLineagePublication<
    'lane,
    'selected,
> {
    pub(super) address: PreparedStableLineageAddress<'lane, 'selected>,
    pub(super) recorded: RecordedOutput,
    pub(super) facts:
        Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
}

impl PreparedStableLineagePublication<'_, '_> {
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.address.identity
    }

    /// The settlement of the generation this alias displaces as the latest
    /// output of its partition: the verified output it restates, when that
    /// belongs to the alias's own occurrence.
    pub(in crate::domain_computation::primary_graph) fn displaced_settlement(
        &self,
    ) -> Option<Arc<RecordedSettlementIdentity>> {
        let displaced = self.address.verified.candidate.settlement_identity();
        (displaced.coordinate().occurrence == self.address.coordinate.occurrence)
            .then(|| Arc::clone(displaced))
    }

    pub(in crate::domain_computation::primary_graph) fn consumed_outputs(
        &self,
    ) -> &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]
    {
        &self.recorded.consumed_outputs
    }
}

pub(in crate::domain_computation::primary_graph) fn prepare_stable_address<
    'lane,
    'selected,
    Binding: 'static,
>(
    owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    verified: VerifiedInputCutoff<'selected>,
    scope: &WorthQueryOperationScopeBinding,
    observation: &ProductBranchObservation,
    coordination: &'lane WorthQueryApplicationBranchCommitCoordination<'lane>,
    admission: &mut InvalidationEditAdmission,
) -> Result<PreparedStableLineageAddress<'lane, 'selected>, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(6)
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    if !coordination.admits(observation) {
        return Err(denial(Kind::PublicationStale));
    }
    let source = SemanticSource {
        runtime_authority: scope.runtime_authority(),
        schema: scope.binding_identity().clone(),
        scope: scope.scope(),
        output_binding: std::any::TypeId::of::<Binding>(),
    };
    if verified.candidate.settlement_identity().source() != &source {
        return Err(denial(Kind::ForeignSource));
    }
    let partition = verified
        .candidate
        .recorded()
        .source_partition_identity
        .expect("an owner-selected input candidate has its exact partition");
    let coordinate = ProductCoordinate {
        occurrence: observation.lifecycle_incarnation(),
        generation: observation.reference_generation().get(),
    };
    let mut lineage = owner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    lineage.drain_cancelled_slots(admission)?;
    // History no retained reader selects is freed before this address extends it.
    lineage.retire_unselected_generations(&source, coordinate.occurrence, admission)?;
    let actual = lineage
        .prior_input_cutoff_candidate::<Binding>(scope, observation, partition, admission)
        .map_err(|stop| match stop {
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
            | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                denial(Kind::WorkBudgetExceeded)
            }
            _ => denial(Kind::RetainedBasisUnavailable),
        })?;
    if !actual
        .as_ref()
        .is_some_and(|actual| same_candidate(actual, &verified.candidate))
    {
        return Err(denial(Kind::PublicationStale));
    }
    admission
        .charge_external_work(
            tree_work::<SemanticSource>(lineage.by_source.len())
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let occurrences = lineage.by_source.get(&source);
    admission
        .charge_external_work(
            tree_work::<worth_runtime_world::facade::ProductBranchIncarnation>(
                occurrences.map_or(0, |rows| rows.len()),
            )
            .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let history = occurrences.and_then(|rows| rows.get(&coordinate.occurrence));
    admission
        .charge_external_work(
            tree_work::<u64>(history.map_or(0, |rows| rows.len()))
                .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
        )
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let slot = history
        .and_then(|rows| rows.get(&coordinate.generation))
        .map_or(0, Vec::len);
    let bytes = arc_bytes::<RecordedSettlementIdentity>()
        .and_then(|bytes| bytes.checked_add(arc_bytes::<OnceLock<RecordedOutput>>()?))
        .ok_or_else(|| denial(Kind::RetentionBudgetExceeded))?;
    admission
        .admit_read_scratch(bytes)
        .map_err(|_| denial(Kind::RetentionBudgetExceeded))?;
    admission
        .charge_external_work(4)
        .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
    let retained_capacity = lineage.retention.reserve(bytes)?;
    let identity = RecordedSettlementIdentity::retain(&source, coordinate, slot);
    let record_cell = Arc::new(OnceLock::new());
    drop(lineage);
    Ok(PreparedStableLineageAddress {
        owner: Arc::clone(owner),
        source,
        coordinate,
        partition,
        identity,
        record_cell,
        expected_record_count: slot,
        verified,
        retained_capacity: Some(retained_capacity),
        _coordination: coordination,
    })
}

fn same_candidate(
    left: &RetainedInputCutoffCandidate,
    right: &RetainedInputCutoffCandidate,
) -> bool {
    Arc::ptr_eq(&left.cell, &right.cell)
}

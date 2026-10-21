//! The state a partitioned computation keeps on the record its attempt
//! published, and the ledger custody that pays for it.
//!
//! Every holder retains the same state and ledger ticket through one Arc.
//! A sole displaced state can yield its ticket only after its final prior is
//! consumed. A fork horizon or another holder preserves the old custody and
//! the successor reserves its distinct state in full. Refusal evicts only
//! the incoming state.

use crate::domain_computation::primary_graph::application_contribution::RetainedComputation;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;
use std::sync::OnceLock;

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_runtime_world::facade::ProductBranchObservation;

mod changes;
use super::recorded_output::RecordedOutput;
use super::ProductCoordinate;
use super::SemanticSource;
use super::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::application_contribution::ComputationPrior;
use crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition;
use crate::domain_computation::primary_graph::application_contribution::PriorAbsence;
use crate::domain_computation::primary_graph::application_contribution::SealedComputationRun;

/// What a record holds of its computation's state.
pub(super) enum RecordedComputation {
    Retained(Arc<RetainedComputation>),
    Absent(PriorAbsence),
}

/// The exact record a producer's run took its prior state from.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PriorComputationRecord {
    cell: Arc<OnceLock<RecordedOutput>>,
}

impl PriorComputationRecord {
    #[cfg(test)]
    pub(super) fn from_recorded_cell_for_test(cell: Arc<OnceLock<RecordedOutput>>) -> Self {
        assert!(
            cell.get().is_some(),
            "the test prior has a real published record"
        );
        Self { cell }
    }
}

impl WorthQueryApplicationOutputLineage {
    /// The latest local record is authoritative, including a typed absence.
    /// With no local record, follow exact captured fork horizons recursively.
    /// Sharing supplies a prior only; the ordinary basis and snapshot checks
    /// still establish whether its calls can be carried. Every lookup is
    /// charged to `admission` before it reads.
    pub(in crate::domain_computation::primary_graph) fn computation_prior_at_address<
        Binding: 'static,
    >(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        observation: &ProductBranchObservation,
        source_partition_identity: [u8; 32],
        edition: InstalledProducerEdition,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<ComputationPrior, CompanionPreflightStop> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: self
                .binding_identity(TypeId::of::<Binding>())
                .expect("admitted binding is installed"),
        };
        let coordinate = ProductCoordinate {
            occurrence: observation.lifecycle_incarnation(),
            generation: observation.reference_generation().get(),
        };
        let Some(cell) = self.computation_cell_in_ancestry(
            &source,
            coordinate,
            source_partition_identity,
            execution,
        )?
        else {
            return Ok(ComputationPrior::new(
                edition,
                Err(PriorAbsence::FirstRun.full_cause()),
                None,
            ));
        };
        let retained = match &cell
            .get()
            .expect("a partition locator references a published record")
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .computation
        {
            RecordedComputation::Retained(handle) => Ok(Arc::clone(handle)),
            RecordedComputation::Absent(absence) => Err(absence.full_cause()),
        };
        Ok(ComputationPrior::new(
            edition,
            retained,
            Some(PriorComputationRecord { cell }),
        ))
    }

    pub(super) fn retain_computation(&self, sealed: SealedComputationRun) -> RecordedComputation {
        let SealedComputationRun {
            state,
            cloned_from,
            tree_memory,
        } = sealed;
        let recorded = RecordedComputation::Retained(Arc::new(state));
        drop(cloned_from);
        drop(tree_memory);
        recorded
    }
}

impl RecordedComputation {
    pub(super) fn shared(&self) -> Self {
        match self {
            Self::Retained(handle) => Self::Retained(Arc::clone(handle)),
            Self::Absent(reason) => Self::Absent(reason.clone()),
        }
    }
}

mod ancestry;

#[cfg(test)]
mod tests;

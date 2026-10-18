//! Selected-field currency and request construction use the original native payer.

use super::IndexAuthority;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexId, DerivedIndexKind, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind as Denial,
    SelectedIndexGenerationAdmissionStop, SelectedIndexReadWork,
};

impl IndexAuthority<'_> {
    /// Ensure one selected field generation from an exact or authentic fork
    /// seed. Population reconstruction is refused. Catalog comparisons, request
    /// copying and allocation use the original execution before work; they do
    /// not consume the caller's separate logical record-selection allowance.
    pub fn ensure_field_index_for_basis(
        &self,
        index: DerivedIndexId,
        basis: &AdmittedRelationalBranchBasis,
        budget: DerivedIndexMaintenanceBudget,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), DerivedIndexMaintenanceDenial> {
        super::request_payment::run(execution, budget, |work| {
            work.restrict_to_field_indexes();
            let branch_bytes = u64::try_from(basis.identity().branch_id().0.len())
                .map_err(|_| Denial::WorkBudgetExceeded)?;
            work.prepare(
                branch_bytes
                    .checked_add(2)
                    .ok_or(Denial::WorkBudgetExceeded)?,
                branch_bytes,
            )?;
            let observation = basis.observation();
            // A catalog hit cannot readmit a foreign basis. Use the same
            // exact-root owner validation as native refresh, including warm hits.
            let _view = self
                .runtime
                .read_truth()
                .project_observation(&observation)
                .map_err(Denial::Basis)?;
            // The definition lookup has one capacity-bounded scalar descent
            // and one Arc retain; no branch payload is hidden in this quote.
            work.prepare(SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK + 1, 0)?;
            let definition = self
                .runtime
                .indexes
                .definition(index)
                .ok_or(Denial::IndexUnavailable(index))?;
            if !matches!(
                &definition.kind,
                DerivedIndexKind::EntityField { .. } | DerivedIndexKind::RelationField { .. }
            ) {
                return Err(Denial::GenerationKindMismatch(index));
            }
            let current = self
                .runtime
                .indexes
                .has_published_generation_for_observation_admitted(
                    index,
                    &observation,
                    |quote, bytes| {
                        let (SelectedIndexReadWork::Operation(units)
                        | SelectedIndexReadWork::OrderedNavigation(units)) = quote;
                        work.prepare(units, bytes)
                    },
                )
                .map_err(|stop| match stop {
                    SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                    SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                        Denial::WorkBudgetExceeded
                    }
                })?;
            if current {
                return Ok(());
            }
            let source_commit_id = observation.commit_id().ok_or(Denial::CommitMismatch)?;
            let branch = observation.identity().branch_id();
            let bytes = branch_bytes
                .checked_add(std::mem::size_of::<DerivedIndexId>() as u64)
                .ok_or(Denial::WorkBudgetExceeded)?;
            work.prepare(
                branch_bytes
                    .checked_add(1)
                    .ok_or(Denial::WorkBudgetExceeded)?,
                bytes,
            )?;
            let request = DerivedIndexBuildRequest {
                source_commit_id,
                branch_id: branch.clone(),
                index_ids: vec![index],
            };
            let prepared = self.prepare_basis_refresh(&request, basis, None, work, execution)?;
            let generations = prepared.publish_admitted(self.runtime, work)?;
            if generations.len() != 1 {
                return Err(Denial::IndexUnavailable(index));
            }
            Ok(())
        })
        .map(|_| ())
    }
}

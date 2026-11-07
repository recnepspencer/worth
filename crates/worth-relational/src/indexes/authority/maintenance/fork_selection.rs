//! Fork cutover carries the source's immutable index selection before exposing the target.
use super::request_payment;
use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::runtime::{IndexDefinitionReadBinding, PreparedGenerationBatch};
use std::sync::Arc;

pub(crate) struct PreparedForkIndexSelection {
    publication: PreparedGenerationBatch,
    _funding: worth_execution::ExecutionMemoryReservation,
}
impl PreparedForkIndexSelection {
    pub(crate) fn publish(self) {
        self.publication.publish();
    }
}
impl IndexDefinitionReadBinding {
    pub(crate) fn prepare_fork_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &BranchId,
        target: &BranchId,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<Option<PreparedForkIndexSelection>, DerivedIndexMaintenanceDenial> {
        let indexes = self.indexing_subsystem();
        if root.commit_id().is_none()
            || indexes.with_definitions(|definitions| definitions.is_empty())
        {
            return Ok(None);
        }
        let (prepared, _) = request_payment::run(
            execution,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: None,
            },
            |work| {
                let envelope = root.canonical_envelope().ok_or(Denial::CommitMismatch)?;
                let schema = root.schema_authority().schema_version();
                let basis = indexes.prepare_basis_selection(
                    target,
                    envelope.commit.commit_id,
                    envelope.commit.version_id,
                    schema,
                    source,
                    envelope.commit.commit_id,
                    envelope.commit.version_id,
                    true,
                    &mut |units, bytes| work.prepare(units, bytes),
                );
                let basis = match basis {
                    Ok(basis) => basis,
                    // A fork does not invent or reconstruct a missing derived predecessor.
                    Err(Denial::ColdReconstructionRequired) => return Ok(None),
                    Err(stop) => return Err(stop),
                };
                indexes.prepare_generation_batch(Vec::new(), Some(basis), |units, bytes| work.prepare(units, bytes))
                .map(Some).map_err(|stop| match stop {
                    crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                    crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => Denial::WorkBudgetExceeded,
                })
            },
        )?;
        let (prepared, funding) = prepared.into_funded_parts();
        Ok(prepared.map(|publication| PreparedForkIndexSelection {
            publication,
            _funding: funding,
        }))
    }
}

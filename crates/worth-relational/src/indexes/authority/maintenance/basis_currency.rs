//! Exact-root selection carry; each read admits its own required generation.
use super::{request_payment, IndexAuthority};
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};

impl IndexAuthority<'_> {
    /// Cost: one branch lookup and, for a still-selected head, one basis lookup.
    /// A batch's superseded selections may already have retired. Native commit
    /// preparation owns every intermediate publication; reads admit their own generation.
    pub fn verify_index_basis_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: &crate::history::data::RelationalCommitReceipt,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: Some(0),
            maximum_derived_rows: None,
        };
        let (_, counts) = request_payment::run(execution, budget, |work| {
            work.lookup(
                self.runtime.history.branch_count(),
                commit.branch_id.0.len() + 1,
            )?;
            if self
                .runtime
                .history
                .branch_cell(&commit.branch_id)
                .and_then(|cell| cell.root())
                .is_none_or(|root| root.commit_id() != Some(commit.commit_id))
            {
                return Ok(());
            }
            self.runtime.indexes.require_basis_selection(
                &commit.branch_id,
                commit.commit_id,
                commit.version_id,
                None,
                |units, bytes| work.prepare(units, bytes),
            )
        })?;
        Ok(counts)
    }

    /// Cost: an exact selection lookup; a genuine fresh fork carries its author's map by identity.
    /// A missing predecessor refuses; this method never reconstructs.
    pub fn carry_index_basis_for_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: Some(0),
            maximum_derived_rows: None,
        };
        let (prepared, counts) = request_payment::run(execution, budget, |work| {
            let observation = basis.observation();
            let projection = self
                .runtime
                .read_truth()
                .project_observation(&observation)
                .map_err(Denial::Basis)?;
            let root = projection
                .selected_root()
                .ok_or(Denial::SnapshotUnavailable)?;
            let Some(envelope) = root.canonical_envelope() else {
                return Ok(None);
            };
            let schema = root.schema_authority().schema_version();
            let branch = observation.identity().branch_id();
            match self.runtime.indexes.require_basis_selection(
                branch,
                envelope.commit.commit_id,
                observation.version_id(),
                Some(schema),
                |units, bytes| work.prepare(units, bytes),
            ) {
                Ok(()) => return Ok(None),
                Err(Denial::ColdReconstructionRequired) => {}
                Err(stop) => return Err(stop),
            }
            if branch == &envelope.branch_context {
                return Err(Denial::ColdReconstructionRequired);
            }
            self.runtime.indexes.require_basis_selection(
                &envelope.branch_context,
                envelope.commit.commit_id,
                observation.version_id(),
                Some(schema),
                |units, bytes| work.prepare(units, bytes),
            )?;
            work.prepare(
                1,
                (branch.0.len()
                    + std::mem::size_of::<crate::runtime::PreparedIndexBasisSelection>())
                    as u64,
            )?;
            let basis = self.runtime.indexes.prepare_basis_selection(
                branch,
                envelope.commit.commit_id,
                observation.version_id(),
                schema,
                &envelope.branch_context,
                envelope.commit.commit_id,
                observation.version_id(),
                &mut |units, bytes| work.prepare(units, bytes),
            )?;
            let mut publication = self
                .runtime
                .indexes
                .prepare_generation_batch(Vec::new(), Some(basis), |units, bytes| {
                    work.prepare(units, bytes)
                })
                .map_err(admission_stop)?;
            publication
                .admit_actual_geometry(|units, bytes| work.prepare(units, bytes))
                .map_err(admission_stop)?;
            Ok(Some(publication))
        })?;
        let (publication, funding) = prepared.into_funded_parts();
        if let Some(mut publication) = publication {
            publication.publish_admitted();
        }
        drop(funding);
        Ok(counts)
    }
}

impl IndexAuthority<'_> {
    /// Explicit recovery/installation. Cost: the installed indexes' complete cold projections.
    pub fn recover_installed_indexes_for_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<crate::indexes::data::DerivedIndexBuildOutcome, DerivedIndexMaintenanceDenial> {
        let observation = basis.observation();
        let source_commit_id = observation
            .commit_id()
            .ok_or(DerivedIndexMaintenanceDenial {
                kind: Denial::CommitMismatch,
                work: DerivedIndexMaintenanceWork::default(),
            })?;
        let request = crate::indexes::data::DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: observation.identity().branch_id().clone(),
            index_ids: self
                .runtime
                .indexes
                .with_definitions(|definitions| definitions.keys().copied().collect()),
        };
        Ok(self.build_for_basis(execution, request, basis))
    }
}

fn admission_stop(
    stop: crate::indexes::data::SelectedIndexGenerationAdmissionStop<Denial>,
) -> Denial {
    match stop {
        crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
        crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
            Denial::WorkBudgetExceeded
        }
    }
}

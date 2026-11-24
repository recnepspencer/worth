//! Exact-root selection carry; each read admits its own required generation.
use super::request_payment;
use crate::branch::{
    basis::AdmittedRelationalBranchBasis, basis_denial::RelationalBranchBasisDenial,
};
use crate::indexes::data::DerivedIndexBuildRequest;
use crate::indexes::data::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::mvcc::observation::RelationalBranchObservation;
use crate::runtime::state::subsystems::{history::HistorySubsystem, indexing::IndexingSubsystem};
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;

pub(super) fn verify_index_basis_for_commit(
    indexes: &IndexingSubsystem,
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    commit: &crate::history::data::RelationalCommitReceipt,
) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: None,
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: None,
    };
    let (_, counts) = request_payment::run(execution, budget, |work| {
        work.lookup(history.branch_count(), commit.branch_id.0.len() + 1)?;
        if history
            .branch_cell(&commit.branch_id)
            .and_then(|cell| cell.root())
            .is_none_or(|root| root.commit_id() != Some(commit.commit_id))
        {
            return Ok(());
        }
        indexes.require_basis_selection(
            &commit.branch_id,
            commit.commit_id,
            commit.version_id,
            None,
            |units, bytes| work.prepare(units, bytes),
        )
    })?;
    Ok(counts)
}

pub(super) fn carry_index_basis_for_basis(
    indexes: &IndexingSubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    mut project: impl FnMut(
        &RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView, RelationalBranchBasisDenial>,
) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: None,
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: None,
    };
    let (prepared, counts) = request_payment::run(execution, budget, |work| {
        let observation = basis.observation();
        let projection = project(&observation).map_err(Denial::Basis)?;
        let root = projection
            .selected_root()
            .ok_or(Denial::SnapshotUnavailable)?;
        let Some(envelope) = root.canonical_envelope() else {
            return Ok(None);
        };
        let schema = root.schema_authority().schema_version();
        let branch = observation.identity().branch_id();
        match indexes.require_basis_selection(
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
        indexes.require_basis_selection(
            &envelope.branch_context,
            envelope.commit.commit_id,
            observation.version_id(),
            Some(schema),
            |units, bytes| work.prepare(units, bytes),
        )?;
        work.prepare(
            1,
            (branch.0.len()
                + std::mem::size_of::<
                    crate::runtime::state::subsystems::indexing::PreparedIndexBasisSelection,
                >()) as u64,
        )?;
        let basis = indexes.prepare_basis_selection(
            branch,
            envelope.commit.commit_id,
            observation.version_id(),
            schema,
            &envelope.branch_context,
            envelope.commit.commit_id,
            observation.version_id(),
            &mut |units, bytes| work.prepare(units, bytes),
        )?;
        let mut publication = indexes
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

pub(super) fn installed_build_request(
    indexes: &IndexingSubsystem,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<DerivedIndexBuildRequest, DerivedIndexMaintenanceDenial> {
    let observation = basis.observation();
    let source_commit_id = observation
        .commit_id()
        .ok_or(DerivedIndexMaintenanceDenial {
            kind: Denial::CommitMismatch,
            work: DerivedIndexMaintenanceWork::default(),
        })?;
    Ok(DerivedIndexBuildRequest {
        source_commit_id,
        branch_id: observation.identity().branch_id().clone(),
        index_ids: indexes.with_definitions(|definitions| definitions.keys().copied().collect()),
    })
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

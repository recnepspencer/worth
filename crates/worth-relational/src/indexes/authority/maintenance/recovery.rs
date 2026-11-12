//! Full projections require an explicit reconstruction scope, never a budget fallback.
use super::{
    changes::ChangedRecords, cold_projection, owner::IndexMaintenanceOwner,
    publication::PreparedRefresh, root_view::IndexRootView, work::MaintenanceWork,
};
use super::{request_payment, root_view};
use crate::indexes::authority::IndexGenerationPublicationBasis;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::indexes::data::*;
use crate::storage::substrate::maintenance::ReconstructionLane;

impl super::IndexAuthority<'_> {
    /// Explicit bounded recovery against an admitted immutable root, including a fork.
    /// Ordinary refresh refuses the same missing state without invoking this door.
    pub fn reconstruct_for_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        budget: crate::indexes::data::DerivedIndexMaintenanceBudget,
    ) -> Result<
        crate::indexes::data::DerivedIndexMaintenanceOutcome,
        crate::indexes::data::DerivedIndexMaintenanceDenial,
    > {
        let (prepared, counts) = ReconstructionLane::enter_reconstruction(|lane| {
            super::request_payment::run(execution, budget, |work| {
                let observation = basis.observation();
                let projection = self
                    .runtime
                    .read_truth()
                    .project_observation(&observation)
                    .map_err(Denial::Basis)?;
                super::basis::validate(&request, &observation, None, &projection, execution)?;
                let after = IndexRootView::from_projection(&projection)?;
                self.maintenance().catalog().routing_for_request(
                    execution,
                    after.root().schema_authority().authority_digest(),
                    after.root().schema_authority().aspect_plans(),
                )?;
                self.maintenance()
                    .prepare_recovery(lane, execution, &request, &after, work)?
                    .publish_admitted(self.runtime, work)
            })
        })?;
        Ok(crate::indexes::data::DerivedIndexMaintenanceOutcome {
            generations: prepared.value,
            work: counts,
        })
    }
}

impl IndexMaintenanceOwner<'_> {
    pub(super) fn prepare_recovery(
        &self,
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &DerivedIndexBuildRequest,
        after: &IndexRootView,
        work: &mut MaintenanceWork,
    ) -> Result<PreparedRefresh, Denial> {
        let records = ChangedRecords::cold(lane, after, execution, work)?;
        let mut entries = Vec::new();
        let mut selected = std::collections::BTreeSet::new();
        for index in &request.index_ids {
            work.ordered::<crate::indexes::data::DerivedIndexId, ()>(selected.len(), 1, 0)?;
            selected.insert(*index);
        }
        let catalog = self.catalog();
        for index in selected {
            work.lookup(catalog.definition_count(), 1)?;
            let definition = catalog
                .definition(index)
                .ok_or(Denial::IndexUnavailable(index))?;
            let rows =
                cold_projection::project(lane, execution, &definition, after, &records, work)?;
            work.grow_vec(&mut entries)?;
            entries.push((index, rows));
        }
        let basis = self.indexes.prepare_reconstruction_basis(
            lane,
            after.root(),
            &request.branch_id,
            &mut |units, bytes| work.prepare(units, bytes),
            self.prepared_catalog(),
        )?;
        Ok(PreparedRefresh {
            publication: IndexGenerationPublicationBasis::new(
                request,
                request.branch_id.clone(),
                after.version_id(),
                after.root().schema_authority().schema_version(),
            ),
            entries,
            reused: Vec::new(),
            basis: Some(basis),
        })
    }
}

impl super::IndexAuthority<'_> {
    /// Explicit bounded reconstruction for a committed version that is no
    /// longer a live branch head. Retention selects its actual immutable root;
    /// the root's envelope must match the requested commit and branch.
    pub fn reconstruct_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let (prepared, counts) = ReconstructionLane::enter_reconstruction(|lane| {
            request_payment::run(execution, budget, |work| {
                work.charge(request.index_ids.len())?;
                let recorded = self
                    .runtime
                    .history
                    .commit_artifact(request.source_commit_id)
                    .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
                let identity = recorded.identity();
                let branch_bytes = request.branch_id.0.len();
                // Compare the authoring name and copy the registry cell identity.
                work.prepare(2 * branch_bytes as u64 + 2, branch_bytes as u64)?;
                if identity.authoring_branch() != &request.branch_id {
                    return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
                }
                let projection = match self.project_registered_retained_commit(
                    execution,
                    &request,
                    identity.version_id(),
                    work,
                )? {
                    Some(projection) => projection,
                    None => {
                        work.prepare(branch_bytes as u64 + 1, branch_bytes as u64)?;
                        self.runtime
                            .read_truth()
                            .try_project_retained_commit(
                                request.source_commit_id,
                                request.branch_id.clone(),
                                identity.version_id(),
                            )
                            .map_err(DerivedIndexMaintenanceDenialKind::Basis)?
                    }
                };
                let root = projection
                    .selected_root()
                    .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
                work.prepare(branch_bytes as u64 + 1, 0)?;
                if root.commit_receipt().is_none_or(|receipt| {
                    receipt.commit_id != request.source_commit_id
                        || receipt.version_id != identity.version_id()
                }) || root
                    .commit_evidence()
                    .is_none_or(|evidence| evidence.authoring_branch() != &request.branch_id)
                    || root.descriptor() != Some(recorded.roots())
                {
                    return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
                }
                self.maintenance().catalog().routing_for_request(
                    execution,
                    root.schema_authority().authority_digest(),
                    root.schema_authority().aspect_plans(),
                )?;
                let after = root_view::IndexRootView::from_projection(&projection)?;
                let prepared = self
                    .maintenance()
                    .prepare_recovery(lane, execution, &request, &after, work)?;
                prepared.publish_admitted(self.runtime, work)
            })
        })?;
        Ok(DerivedIndexMaintenanceOutcome {
            generations: prepared.value,
            work: counts,
        })
    }
}

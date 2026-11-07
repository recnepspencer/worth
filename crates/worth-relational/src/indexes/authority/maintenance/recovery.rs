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
                super::basis::validate(&request, &observation, None, &projection)?;
                let after = IndexRootView::from_projection(&projection)?;
                self.runtime.indexes.install_routing(
                    after.root().schema_authority().authority_digest(),
                    after.root().schema_authority().aspect_plans(),
                );
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
        for index in selected {
            work.prepare(
                crate::indexes::data::SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK + 1,
                0,
            )?;
            let definition = self
                .indexes
                .definition(index)
                .ok_or(Denial::IndexUnavailable(index))?;
            let rows = cold_projection::project(execution, &definition, after, &records, work)?;
            work.grow_vec(&mut entries)?;
            entries.push((index, rows));
        }
        let basis = self.indexes.prepare_reconstruction_basis(
            lane,
            after.root(),
            &request.branch_id,
            work.publication_is_deferred(),
            &mut |units, bytes| work.prepare(units, bytes),
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
                    .recorded_commit_envelope(request.source_commit_id)
                    .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
                if recorded.branch_context != request.branch_id {
                    return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
                }
                let projection = self
                    .runtime
                    .read_truth()
                    .try_project_retained_commit(
                        request.source_commit_id,
                        request.branch_id.clone(),
                        recorded.commit.version_id,
                    )
                    .map_err(DerivedIndexMaintenanceDenialKind::Basis)?;
                let root = projection
                    .selected_root()
                    .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
                if root.commit_id() != Some(request.source_commit_id)
                    || root
                        .canonical_envelope()
                        .is_none_or(|envelope| envelope.branch_context != request.branch_id)
                {
                    return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
                }
                self.runtime.indexes.install_routing(
                    root.schema_authority().authority_digest(),
                    root.schema_authority().aspect_plans(),
                );
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

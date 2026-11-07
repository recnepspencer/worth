//! Native pipeline entrance: authentic roots and their one sealed shared change.
use super::{
    candidate_publication::{CandidateIndexPreparation, PreparedCandidateIndexPublication},
    owner::IndexMaintenanceOwner,
    request_payment,
    root_view::IndexRootView,
};
use crate::branch::RelationalBranchRoot;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::runtime::IndexDefinitionReadBinding;
use crate::storage::substrate::maintenance::CommitChange;
use std::sync::Arc;

impl IndexDefinitionReadBinding {
    /// Cost: the sealed change and reached installed indexes, plus keyed selection paths.
    /// No public candidate or full runtime exists or is required at this boundary.
    pub(crate) fn prepare_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &Arc<RelationalBranchRoot>,
        successor: &Arc<RelationalBranchRoot>,
        change: &Arc<CommitChange>,
    ) -> Result<Option<PreparedCandidateIndexPublication>, DerivedIndexMaintenanceDenial> {
        if change.schema_changed() && predecessor.commit_id().is_some() {
            return crate::storage::substrate::maintenance::ReconstructionLane::recover(
                execution,
                |lane, _| {
                    self.prepare_index_publication(
                        execution,
                        predecessor,
                        successor,
                        change,
                        Some(lane),
                    )
                },
            );
        }
        self.prepare_index_publication(execution, predecessor, successor, change, None)
    }

    fn prepare_index_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &Arc<RelationalBranchRoot>,
        successor: &Arc<RelationalBranchRoot>,
        change: &Arc<CommitChange>,
        reconstruction: Option<&crate::storage::substrate::maintenance::ReconstructionLane<'_>>,
    ) -> Result<Option<PreparedCandidateIndexPublication>, DerivedIndexMaintenanceDenial> {
        crate::execution::check_request(execution).map_err(|cause| {
            DerivedIndexMaintenanceDenial {
                kind: Denial::Execution(cause),
                work: Default::default(),
            }
        })?;
        let indexes = self.indexing_subsystem();
        if indexes.with_definitions(|definitions| definitions.is_empty()) {
            return Ok(None);
        }
        let owner = IndexMaintenanceOwner { indexes: &indexes };
        let (prepared, counts) = request_payment::run(
            execution,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: if reconstruction.is_some() {
                    None
                } else {
                    Some(0)
                },
                maximum_derived_rows: None,
            },
            |work| {
                if successor
                    .commit_change()
                    .is_none_or(|actual| !Arc::ptr_eq(actual, change))
                    || successor.commit_id() != Some(change.commit_id())
                {
                    return Err(Denial::CommitMismatch);
                }
                let envelope = successor
                    .canonical_envelope()
                    .ok_or(Denial::CommitMismatch)?;
                super::basis::validate_root_predecessor(envelope, predecessor)?;
                work.prepare(
                    envelope.branch_context.0.len() as u64,
                    envelope.branch_context.0.len() as u64,
                )?;
                let mut request = DerivedIndexBuildRequest {
                    source_commit_id: change.commit_id(),
                    branch_id: envelope.branch_context.clone(),
                    index_ids: Vec::new(),
                };
                if reconstruction.is_some() {
                    // A sealed schema transition is the explicit cold trust boundary.
                    indexes.install_routing(
                        successor.schema_authority().schema_version(),
                        successor.schema_authority().aspect_plans(),
                    );
                    indexes.with_definitions(|definitions| -> Result<(), Denial> {
                        for index in definitions.keys() {
                            work.grow_vec(&mut request.index_ids)?;
                            request.index_ids.push(*index);
                        }
                        Ok(())
                    })?;
                } else {
                    work.restrict_to_routed_indexes();
                }
                work.defer_publication();
                let before = IndexRootView::new(Arc::clone(predecessor));
                let after = IndexRootView::new(Arc::clone(successor));
                let refresh = owner.prepare_refresh(
                    execution,
                    &request,
                    if reconstruction.is_some() {
                        None
                    } else {
                        Some(&before)
                    },
                    &after,
                    work,
                )?;
                CandidateIndexPreparation::prepare(&indexes, successor, refresh, work)
            },
        )?;
        let (prepared, funding) = prepared.into_funded_parts();
        Ok(Some(PreparedCandidateIndexPublication {
            prepared,
            funding,
            work: counts,
        }))
    }
}

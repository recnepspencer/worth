//! Native pipeline entrance: authentic roots and their one sealed shared change.
use super::{
    candidate_publication::PreparedCandidateIndexPublication, owner::IndexMaintenanceOwner,
    request_payment, root_view::IndexRootView,
};
use crate::contracts::indexes::data::maintenance::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionWorkOwnerPort as _;
use crate::contracts::storage::substrate::maintenance::work::ChangeWorkOwnerPort as _;
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionReadBinding, PreparedRelationalIndexDefinitions,
};
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::maintenance::CommitChange;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexBuildRequest;

impl IndexDefinitionReadBinding {
    /// Cost: the sealed change and reached installed indexes, plus keyed selection paths.
    /// No public candidate or full runtime exists or is required at this boundary.
    pub fn prepare_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &Arc<RelationalBranchRoot>,
        successor: &Arc<RelationalBranchRoot>,
        change: &Arc<CommitChange>,
        definitions: Option<&PreparedRelationalIndexDefinitions>,
        maintenance_budget: Option<DerivedIndexMaintenanceBudget>,
    ) -> Result<Option<PreparedCandidateIndexPublication>, DerivedIndexMaintenanceDenial> {
        crate::contracts::storage::substrate::maintenance::work::ChangeWork::run_phase(
            execution,
            "index_maintenance",
            |_| {
                if change.schema_changed() && predecessor.commit_id().is_some() {
                    self.prepare_schema_transition(
                        execution,
                        predecessor,
                        successor,
                        change,
                        definitions,
                        maintenance_budget,
                    )
                } else {
                    self.prepare_index_publication(
                        execution,
                        predecessor,
                        successor,
                        change,
                        None,
                        definitions,
                        maintenance_budget,
                    )
                }
            },
        )
    }

    pub(super) fn prepare_index_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &Arc<RelationalBranchRoot>,
        successor: &Arc<RelationalBranchRoot>,
        change: &Arc<CommitChange>,
        mut reconstruction: Option<(
            &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
            &mut crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionWork<'_, '_>,
        )>,
        definitions: Option<&PreparedRelationalIndexDefinitions>,
        maintenance_budget: Option<DerivedIndexMaintenanceBudget>,
    ) -> Result<Option<PreparedCandidateIndexPublication>, DerivedIndexMaintenanceDenial> {
        crate::contracts::execution::request_custody::check_request(execution).map_err(
            |cause| DerivedIndexMaintenanceDenial {
                kind: Denial::Execution(cause),
                work: Default::default(),
            },
        )?;
        let indexes = self.indexing_subsystem();
        let owner = IndexMaintenanceOwner {
            indexes: &indexes,
            definitions,
        };
        let catalog = owner.catalog();
        if catalog.is_empty() {
            return Ok(None);
        }
        let mut budget = maintenance_budget.unwrap_or(DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: None,
            maximum_derived_rows: None,
        });
        if reconstruction.is_none() {
            // A caller allowance never grants the ordinary lane cold authority.
            budget.maximum_cold_record_slots = Some(0);
        }
        let (prepared, counts) = request_payment::run(execution, budget, |work| {
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
                for index in catalog.index_ids() {
                    work.grow_vec(&mut request.index_ids)?;
                    request.index_ids.push(index);
                }
            } else {
                work.restrict_to_routed_indexes();
            }
            let before = IndexRootView::new(Arc::clone(predecessor));
            let after = IndexRootView::new(Arc::clone(successor));
            // Descriptive route compilation grants no population-reconstruction door.
            // Ordinary installed catalogs must already carry their schema routes.
            let routes = if definitions.is_some() || reconstruction.is_some() {
                catalog.routing_for_request(
                    execution,
                    successor.schema_authority().authority_digest(),
                    successor.schema_authority().aspect_plans(),
                )?
            } else {
                owner.routes(successor)?
            };
            let inputs = if reconstruction.is_some() {
                None
            } else {
                Some(owner.refresh_inputs(execution, &request, Some(&before), &after, work)?)
            };
            let previous = inputs
                .as_ref()
                .and_then(|inputs| inputs.inherited.as_ref())
                .map(|basis| basis.selection_state());
            let work_cell = std::cell::RefCell::new(work);
            let mut maintainer = super::maintained_selection::IndexSelectionMaintainer {
                owner: &owner,
                execution,
                before: &before,
                after: &after,
                request: &request,
                native: &work_cell,
                routes,
                inputs,
                prepared: None,
            };
            let mut prepare = |units, bytes| {
                work_cell
                    .borrow_mut()
                    .prepare(units, bytes)
                    .map_err(|denial| match denial {
                        Denial::Execution(cause) => {
                            crate::contracts::storage::substrate::maintenance::refusal::MaintenanceRefusal::Execution(
                                cause,
                            )
                        }
                        _ => crate::contracts::storage::substrate::maintenance::refusal::MaintenanceRefusal::Execution(
                            crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
                        ),
                    })
            };
            let state = crate::contracts::storage::substrate::maintenance::work::ChangeWork::run_with_payer(
                &mut prepare,
                |shared| {
                    if let Some((lane, _)) = reconstruction.as_mut() {
                        let mut cold_work =
                            crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionWork::from_change_work(
                                shared,
                            );
                        worth_relational_branch::internal_port::branch::maintenance::Maintained::cold(
                            &mut maintainer,
                            lane,
                            &mut cold_work,
                        )
                    } else {
                        worth_relational_branch::internal_port::branch::maintenance::apply_change(
                            &mut maintainer,
                            previous.as_ref(),
                            change,
                            shared,
                        )
                    }
                },
            )?;
            maintainer.finish(state)
        })?;
        let (prepared, funding) = prepared.into_funded_parts();
        Ok(Some(PreparedCandidateIndexPublication {
            prepared,
            funding,
            metadata: None,
            work: counts,
        }))
    }
}

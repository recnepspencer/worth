//! A verified schema transition is the sole publication-time reconstruction entrance.
use super::candidate_publication::PreparedCandidateIndexPublication;
use crate::contracts::indexes::data::maintenance::{
    DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionReadBinding, PreparedRelationalIndexDefinitions,
};
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::maintenance::CommitChange;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;

impl IndexDefinitionReadBinding {
    pub(super) fn prepare_schema_transition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        predecessor: &Arc<RelationalBranchRoot>,
        successor: &Arc<RelationalBranchRoot>,
        change: &Arc<CommitChange>,
        definitions: Option<&PreparedRelationalIndexDefinitions>,
        maintenance_budget: Option<
            crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceBudget,
        >,
    ) -> Result<Option<PreparedCandidateIndexPublication>, DerivedIndexMaintenanceDenial> {
        if !change.schema_changed()
            || predecessor.commit_id().is_none()
            || successor.commit_id() != Some(change.commit_id())
            || successor
                .commit_change()
                .is_none_or(|actual| !Arc::ptr_eq(actual, change))
            || predecessor.schema_authority().authority_digest()
                == successor.schema_authority().authority_digest()
        {
            return Err(DerivedIndexMaintenanceDenial {
                kind: Denial::CommitMismatch,
                work: Default::default(),
            });
        }
        super::basis::validate_root_predecessor(
            successor
                .canonical_envelope()
                .ok_or(DerivedIndexMaintenanceDenial {
                    kind: Denial::CommitMismatch,
                    work: Default::default(),
                })?,
            predecessor,
        )
        .map_err(|kind| DerivedIndexMaintenanceDenial {
            kind,
            work: Default::default(),
        })?;
        ReconstructionLane::enter_reconstruction(|lane| {
            lane.recover(execution, |lane, work| {
                self.prepare_index_publication(
                    execution,
                    predecessor,
                    successor,
                    change,
                    Some((lane, work)),
                    definitions,
                    maintenance_budget,
                )
            })
        })
    }
}

/// Admit `budget` against Native's exact staged successor cold-reconstruction
/// projection under the blessed schema-transition reconstruction boundary.
pub fn admit_home_target_cold_reconstruction_budget(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: super::cold_preflight::StagedHomeTargetSuccessor,
    definitions: Option<&PreparedRelationalIndexDefinitions>,
    fallback_catalog: &crate::runtime::state::subsystems::indexing::IndexDefinitionCatalog,
    budget: crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceBudget,
) -> Result<
    crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceWork,
    DerivedIndexMaintenanceDenial,
> {
    if !super::cold_preflight::target_catalog_is_non_empty(definitions, fallback_catalog) {
        return Ok(Default::default());
    }
    ReconstructionLane::enter_reconstruction(|lane| {
        super::cold_preflight::admit_in_lane(
            lane,
            execution,
            staged,
            definitions,
            fallback_catalog,
            budget,
        )
    })
}

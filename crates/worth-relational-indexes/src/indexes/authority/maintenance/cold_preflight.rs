//! Native-owned exact pre-intent budget admission for Home program transition
//! target cold index reconstruction.
//!
//! Runs Native's actual `ChangedRecords::cold` + `cold_projection::project` +
//! per-index `CandidateIndexPreparation::prepare` work accounting directly
//! against the staged successor state and prepared target index catalog
//! without writing to Store, consuming generation IDs, or publishing anything.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_storage::internal_port::storage::overlay::PartitionState;

use super::changes::ChangedRecords;
use super::cold_projection;
use super::request_payment;
use super::root_view::{IndexRootView, StagedSuccessorOverlay};
use crate::contracts::identity::data::primitives::{PartitionId, VersionId};
use crate::contracts::indexes::data::definition::DerivedIndexId;
use crate::contracts::indexes::data::maintenance::{
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionCatalog, PreparedRelationalIndexDefinitions,
};

/// In-memory staged successor view for pre-intent cold-reconstruction
/// dry-run admission.
pub struct StagedHomeTargetSuccessor {
    pub predecessor_root: Arc<RelationalBranchRoot>,
    pub version_id: VersionId,
    pub schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
    pub staged_partitions: BTreeMap<PartitionId, PartitionState>,
    pub entity_slot_count: usize,
    pub relation_slot_count: usize,
    pub region_count: usize,
}

/// Returns `true` when the selected target index catalog is non-empty and
/// cold reconstruction must run for a schema transition.
pub fn target_catalog_is_non_empty(
    definitions: Option<&PreparedRelationalIndexDefinitions>,
    fallback_catalog: &IndexDefinitionCatalog,
) -> bool {
    let catalog = definitions.map_or(fallback_catalog, |prepared| {
        prepared.captured.catalog.as_ref()
    });
    !catalog.is_empty()
}

/// Run Native's exact cold-reconstruction projection and budget accounting on
/// `staged` against `definitions` (or `fallback_catalog`) inside an already
/// admitted schema-transition `ReconstructionLane`.
///
/// Guarantees bit-for-bit agreement with `prepare_schema_transition` /
/// `prepare_recovery` / `CandidateIndexPreparation::prepare` on
/// `cold_record_slots`, `record_reads`, `derived_rows`,
/// `generation_publications_reserved`, and `work_units`.
pub(super) fn admit_in_lane(
    lane: &ReconstructionLane<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    staged: StagedHomeTargetSuccessor,
    definitions: Option<&PreparedRelationalIndexDefinitions>,
    fallback_catalog: &IndexDefinitionCatalog,
    budget: DerivedIndexMaintenanceBudget,
) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
    let catalog = definitions.map_or(fallback_catalog, |prepared| {
        prepared.captured.catalog.as_ref()
    });
    if catalog.is_empty() {
        return Ok(DerivedIndexMaintenanceWork::default());
    }
    let mut all_partitions: BTreeSet<PartitionId> =
        staged.predecessor_root.partition_ids_iter().collect();
    all_partitions.extend(staged.staged_partitions.keys().copied());
    let after = IndexRootView::staged_successor(
        staged.predecessor_root,
        staged.version_id,
        StagedSuccessorOverlay {
            schema_authority: staged.schema_authority,
            staged_partitions: staged.staged_partitions,
            all_partitions,
            entity_slot_count: staged.entity_slot_count,
            relation_slot_count: staged.relation_slot_count,
            region_count: staged.region_count,
        },
    );
    request_payment::run(execution, budget, |work| {
        let records = ChangedRecords::cold(lane, &after, execution, work)?;
        let mut selected: BTreeSet<DerivedIndexId> = BTreeSet::new();
        for index in catalog.index_ids() {
            work.ordered::<DerivedIndexId, ()>(selected.len(), 1, 0)?;
            selected.insert(index);
        }
        for &index in &selected {
            work.lookup(catalog.definition_count(), 1)?;
            let definition = catalog
                .definition(index)
                .ok_or(Denial::IndexUnavailable(index))?;
            let _rows =
                cold_projection::project(lane, execution, &definition, &after, &records, work)?;
        }
        work.charge(selected.len())?;
        work.reserve_generation_publications(selected.len());
        Ok(())
    })
    .map(|(_prepared, counts)| counts)
}

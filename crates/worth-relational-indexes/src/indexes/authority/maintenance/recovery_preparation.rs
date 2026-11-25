//! Build reconstructed index rows and basis bindings under the selected reconstruction lane.
use super::{
    changes::ChangedRecords, cold_projection, owner::IndexMaintenanceOwner,
    publication::PreparedRefresh, root_view::IndexRootView, work::MaintenanceWork,
};
use crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind as Denial;
use crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use crate::indexes::authority::IndexGenerationPublicationBasis;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexBuildRequest;

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
            work.ordered::<crate::contracts::indexes::data::definition::DerivedIndexId, ()>(
                selected.len(),
                1,
                0,
            )?;
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

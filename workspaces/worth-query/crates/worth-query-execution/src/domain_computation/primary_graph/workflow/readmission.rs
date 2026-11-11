//! Structural cold workflow DATA admission on an actual selected owner source.
mod definitions;
mod instances;
mod population;
mod proposals;
mod transitions;
use super::schema::WorthQueryWorkflowLayout;
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::read_fields::ReadFields,
    application_discovery::WorthQueryWorkflowReadDenial, WorthQueryPrimaryGraphInstallationDenial,
};
use population::WorkflowPopulation;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalRuntime,
    snapshots::{
        RelationalSnapshotAdmissionDenial, RelationalSnapshotReleaseDenial, SnapshotHandle,
    },
};
use worth_runtime_world::facade::ProductBranchObservation;

#[derive(Debug)]
pub enum QueryHomeWorkflowReadmissionDenial {
    Source(WorthQueryPrimaryGraphInstallationDenial),
    EntityPopulation(worth_relational::facade::runtime::EntityKindTruthReadLimitExceeded),
    RelationPopulation(worth_relational::facade::runtime::RelationKindTruthReadDenial),
    Record(WorthQueryWorkflowReadDenial),
    Start(super::instance::start_provenance::StartProvenanceReadDenial),
    Snapshot(RelationalSnapshotAdmissionDenial),
    SnapshotRelease {
        snapshot: SnapshotHandle,
        cause: RelationalSnapshotReleaseDenial,
        preceding: Option<Box<Self>>,
    },
    CorruptRecord {
        entity: EntityId,
        kind: KindId,
    },
    UnattributedProgram {
        entity: EntityId,
        revision: CanonicalDigestId,
    },
    CurrentProgramMismatch {
        entity: EntityId,
    },
}
use QueryHomeWorkflowReadmissionDenial as Denial;

pub(in crate::domain_computation::primary_graph) fn readmit(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryWorkflowLayout,
    branch: &ProductBranchObservation,
    current: CanonicalDigestId,
    history: &BTreeSet<CanonicalDigestId>,
) -> Result<(), Denial> {
    let basis = branch.basis().relational_basis();
    let view = native.read_truth().project_observation(&basis.observation()).map_err(|cause| Denial::Source(
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial::new(
            crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
            format!("workflow source observation refused: {cause:?}"),
        )))?;
    let population = WorkflowPopulation::read(&view, execution, layout)?;
    if population.is_empty() {
        return Ok(());
    }
    let snapshot = native
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .map_err(Denial::Snapshot)?;
    let result = (|| {
        let fields = ReadFields::new(native, execution, &snapshot).map_err(Denial::Record)?;
        let definitions = definitions::read(&fields, layout, &population, current, history)?;
        let context = WorkflowSourceRecords {
            fields,
            layout,
            population,
            definitions,
            branch,
            current,
            history,
        };
        instances::readmit(&context)?;
        transitions::readmit(&context)?;
        proposals::readmit(&context)
    })();
    match native.snapshots().release_snapshot(&snapshot) {
        Ok(_) => result,
        Err(cause) => Err(Denial::SnapshotRelease {
            snapshot,
            cause,
            preceding: result.err().map(Box::new),
        }),
    }
}

struct WorkflowSourceRecords<'a, 'e, 'r> {
    fields: ReadFields<'a, 'e, 'r>,
    layout: &'a WorthQueryWorkflowLayout,
    population: WorkflowPopulation,
    definitions: BTreeMap<EntityId, super::definition::RecordedWorkflowDefinition>,
    branch: &'a ProductBranchObservation,
    current: CanonicalDigestId,
    history: &'a BTreeSet<CanonicalDigestId>,
}
impl WorkflowSourceRecords<'_, '_, '_> {
    fn corrupt(&self, entity: EntityId, kind: KindId) -> Denial {
        Denial::CorruptRecord { entity, kind }
    }
    fn revision(
        &self,
        entity: EntityId,
        kind: KindId,
        field: &worth_foundational::facade::AspectFieldLocator,
    ) -> Result<CanonicalDigestId, Denial> {
        let text = self
            .fields
            .text(entity, kind, field)
            .map_err(Denial::Record)?;
        let text = std::str::from_utf8(text.bytes()).map_err(|_| self.corrupt(entity, kind))?;
        CanonicalDigestId::parse_hex(text)
            .filter(|digest| digest.render_hex() == text)
            .ok_or_else(|| self.corrupt(entity, kind))
    }
    fn attribute(&self, entity: EntityId, revision: CanonicalDigestId) -> Result<(), Denial> {
        if self.history.contains(&revision) {
            Ok(())
        } else {
            Err(Denial::UnattributedProgram { entity, revision })
        }
    }
}

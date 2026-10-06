use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_foundational::facade::{CanonicalFieldPath, ContractValidatedAspectValueView};
use worth_query::facade::runtime::WorthQueryPrimaryGraphSourceProjection;
use worth_query::facade::{domain, foundation, runtime};

#[derive(Default)]
pub struct SourceObservations {
    exact_record_reads: AtomicUsize,
    full_target_reads: AtomicUsize,
}

impl SourceObservations {
    pub fn exact_record_reads(&self) -> usize {
        self.exact_record_reads.load(Ordering::SeqCst)
    }

    pub fn full_target_reads(&self) -> usize {
        self.full_target_reads.load(Ordering::SeqCst)
    }
}

pub struct IntentSourceProjection {
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    observations: Arc<SourceObservations>,
}

impl IntentSourceProjection {
    pub fn new(
        record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        observations: Arc<SourceObservations>,
    ) -> Self {
        Self {
            record,
            observations,
        }
    }
}

impl WorthQueryPrimaryGraphSourceProjection for IntentSourceProjection {
    fn project_live_target(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        _target: &runtime::WorthQueryLiveArtifactTarget,
    ) -> Vec<foundation::WorthQueryEntity> {
        self.observations
            .full_target_reads
            .fetch_add(1, Ordering::SeqCst);
        project_record(graph, self.record).into_iter().collect()
    }

    fn project_granular_scope(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        _target: &runtime::WorthQueryLiveArtifactTarget,
        scope: &domain::WorthQueryMaintenanceScope,
        basis: &runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
        let requested = match scope {
            domain::WorthQueryMaintenanceScope::ExactSourceRecord {
                partition_id,
                local_slot,
                generation,
            } => worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                *partition_id,
                *local_slot,
                *generation,
            ),
            // The full scope of this live target is its one record.
            domain::WorthQueryMaintenanceScope::WholeLogicalGraph => self.record,
            domain::WorthQueryMaintenanceScope::SourcePartition(_) => {
                return Err(foundation::WorthQueryWorkspaceError::new(
                    "the temporal certification source admits records or its full scope only",
                ))
            }
        };
        if requested != self.record {
            return Ok(Vec::new());
        }
        self.observations
            .exact_record_reads
            .fetch_add(1, Ordering::SeqCst);
        Ok(project_record_at_basis(graph, basis, requested)?
            .into_iter()
            .collect())
    }
}

fn project_record(
    graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> Option<foundation::WorthQueryEntity> {
    graph.with_runtime(|runtime| {
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).ok()?;
        let observation = basis.observation();
        project_observation_record(runtime, &observation, record)
    })
}

fn project_record_at_basis(
    graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
    basis: &runtime::WorthQueryGranularSourceReadBasis,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> Result<Option<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
    graph
        .relational_bridge_source()
        .read_retained_entity_aspect_state(basis.snapshot(), basis.branch(), record)
        .map(|state| state.and_then(|state| project_aspect_state(record, &state)))
        .map_err(|error| foundation::WorthQueryWorkspaceError::new(error.to_string()))
}

fn project_observation_record(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    observation: &worth_relational::facade::branch::RelationalBranchObservation,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> Option<foundation::WorthQueryEntity> {
    let read_view = runtime.read_truth().read_observation(observation).ok()?;
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::new(record.partition_id()),
        record.local_slot(),
        record.generation(),
    );
    let authoritative = read_view
        .entities()
        .iter()
        .find(|candidate| candidate.entity_id == entity)?;
    project_aspect_state(record, authoritative.authoritative_aspect_state.as_ref()?)
}

fn project_aspect_state(
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    state: &worth_foundational::facade::AuthoritativeRecordAspectState,
) -> Option<foundation::WorthQueryEntity> {
    let mut fields = BTreeMap::new();
    for (aspect, value) in state.aspects().entries() {
        let ContractValidatedAspectValueView::Struct(value) = value.view() else {
            continue;
        };
        let aspect = worth_foundational::facade::FieldKey::new(aspect.as_str().to_owned())?;
        fields.extend(value.fields().filter_map(|(field, value)| {
            CanonicalFieldPath::new(vec![aspect.clone(), field.clone()])
                .map(|path| (path, value.clone()))
        }));
    }
    Some(foundation::WorthQueryEntity::from_native_field_values(
        foundation::WorthQueryEntityIdentity::from_bridge_record_projection(record),
        fields,
    ))
}

use std::collections::{BTreeMap, BTreeSet};

use super::comparison::ObservedSupplyChainState;
use super::handles::SupplyChainSemanticHandles;
use super::observation_debug::{
    debug_booking, debug_class, debug_hazard, debug_posture, debug_region, debug_result,
    debug_status,
};
use super::production_world::ProductionSeededSupplyChainWorld;
use super::program::CompiledSupplyChainProgram;
use super::schema::{
    CargoLotRecord, EntityRecord, InspectionRecord, PortCallRecord, PortRecord, RelationEdge,
    TerminalRecord, VesselRecord, VoyageRecord,
};
use super::semantic_key::{BranchLabel, EntityKey, EntityKind, RelationKey};
use worth_foundational::facade::{
    AspectKey, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    InternedString,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::runtime::{RelationalReadView, RelationalRuntime};
use worth_relational::facade::snapshots::SnapshotHandle;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ObservationError {
    SnapshotUnavailable,
    BranchBasis(worth_relational::facade::branch::RelationalBranchBasisDenial),
    UnknownBranch(worth_relational::facade::history::BranchId),
    UnknownEntityIdentity(EntityId),
    UnknownRelationIdentity(RelationId),
    MissingEntity(EntityKey),
    MissingRelation(RelationKey),
    WrongEntityKind {
        key: EntityKey,
        observed: KindId,
    },
    WrongRelationKind {
        key: RelationKey,
        observed: KindId,
    },
    UnknownRelationEndpoint(RelationKey),
    MissingAspect {
        entity: EntityKey,
        name: String,
    },
    InvalidAspect {
        entity: EntityKey,
        name: String,
        detail: String,
    },
    UnsupportedSchemaVersion(worth_relational::facade::schema::SchemaVersionId),
}

pub(crate) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
) -> Result<ObservedSupplyChainState, ObservationError> {
    let observation = world.basis.observation();
    if observation.identity().runtime_instance_id() != world.handles.branch.runtime_instance_id
        || observation.identity().branch_id() != &world.handles.branch.branch_id
    {
        return Err(ObservationError::SnapshotUnavailable);
    }
    observe_observation(
        execution,
        &world.program,
        &world.handles,
        &world.runtime,
        &observation,
    )
}

/// Observe the immutable root carried by an owner-admitted branch observation.
pub(crate) fn observe_observation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    runtime: &RelationalRuntime,
    observation: &worth_relational::facade::branch::RelationalBranchObservation,
) -> Result<ObservedSupplyChainState, ObservationError> {
    if observation.identity().runtime_instance_id() != handles.branch.runtime_instance_id {
        return Err(ObservationError::SnapshotUnavailable);
    }
    let view = runtime
        .read_truth()
        .read_observation(execution, observation)
        .map_err(ObservationError::BranchBasis)?;
    let schema = runtime
        .read_truth()
        .observation_schema_version(observation)
        .map_err(ObservationError::BranchBasis)
        .and_then(schema_version)?;
    assemble_observation(
        program,
        handles,
        runtime,
        observation.identity().branch_id(),
        schema,
        &view,
    )
}

/// Observe one owner-selected branch snapshot without rebuilding a world or
/// consulting the global latest partition set.  The caller must supply the
/// exact branch-qualified handle issued by `VisibilityAuthority`.
pub(crate) fn observe_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<ObservedSupplyChainState, ObservationError> {
    if snapshot.runtime_instance_id() != handles.branch.runtime_instance_id
        || snapshot.branch_id() != &handles.branch.branch_id
    {
        return Err(ObservationError::SnapshotUnavailable);
    }
    let Some(view) = runtime.read_truth().read_snapshot(snapshot, execution) else {
        return Err(ObservationError::SnapshotUnavailable);
    };
    let schema = observe_schema_version(runtime, snapshot)?;
    assemble_observation(
        program,
        handles,
        runtime,
        snapshot.branch_id(),
        schema,
        &view,
    )
}

fn assemble_observation(
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    runtime: &RelationalRuntime,
    branch_id: &worth_relational::facade::history::BranchId,
    schema: super::schema::SchemaVersion,
    view: &RelationalReadView,
) -> Result<ObservedSupplyChainState, ObservationError> {
    let entities = observe_entities(program, handles, view)?;
    let relations = observe_relations(handles, view)?;
    let branch = branch_label(branch_id)?;
    let parent = runtime
        .branch_reference_state(branch_id)
        .and_then(|state| state.fork_source_branch_id().cloned())
        .map(|branch_id| branch_label(&branch_id))
        .transpose()?;
    let lineage = parent.map_or_else(|| vec![branch], |parent| vec![parent, branch]);
    Ok(ObservedSupplyChainState {
        schema,
        relation_vector: relations.values().copied().collect(),
        entities,
        relations,
        absent_entities: BTreeSet::new(),
        absent_relations: BTreeSet::new(),
        branch,
        parent,
        lineage,
        accepted: Vec::new(),
        history: Vec::new(),
    })
}

fn observe_schema_version(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<super::schema::SchemaVersion, ObservationError> {
    let observed = runtime
        .read_truth()
        .snapshot_schema_version(snapshot)
        .ok_or(ObservationError::SnapshotUnavailable)?;
    schema_version(observed)
}

fn schema_version(
    observed: worth_relational::facade::schema::SchemaVersionId,
) -> Result<super::schema::SchemaVersion, ObservationError> {
    match observed.0 {
        1 => Ok(super::schema::SchemaVersion::V1),
        2 => Ok(super::schema::SchemaVersion::V2),
        _ => Err(ObservationError::UnsupportedSchemaVersion(observed)),
    }
}

fn branch_label(
    branch_id: &worth_relational::facade::history::BranchId,
) -> Result<BranchLabel, ObservationError> {
    match branch_id.0.as_str() {
        "main" => Ok(BranchLabel::Operating),
        "storm" => Ok(BranchLabel::Storm),
        "maintenance" => Ok(BranchLabel::Maintenance),
        "customs" => Ok(BranchLabel::Customs),
        "medical-hold" => Ok(BranchLabel::MedicalHold),
        "southpoint-expansion" => Ok(BranchLabel::SouthpointExpansion),
        "competing-arrival" => Ok(BranchLabel::CompetingArrival),
        "inspection" => Ok(BranchLabel::Inspection),
        "rewire" => Ok(BranchLabel::Rewire),
        "hazard-v2" | "hazard-v2-secondary" => Ok(BranchLabel::HazardV2),
        _ => Err(ObservationError::UnknownBranch(branch_id.clone())),
    }
}

fn observe_entities(
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    view: &RelationalReadView,
) -> Result<BTreeMap<EntityKey, EntityRecord>, ObservationError> {
    let mut entities = BTreeMap::new();
    for record in view.entities() {
        let Some(key) = handles.entity_key(record.entity_id) else {
            return Err(ObservationError::UnknownEntityIdentity(record.entity_id));
        };
        let expected_kind = key.kind;
        if record.kind.kind_id != super::program::entity_kind_id(expected_kind) {
            return Err(ObservationError::WrongEntityKind {
                key,
                observed: record.kind.kind_id,
            });
        }
        let Some(state) = record.authoritative_aspect_state.as_ref() else {
            return Err(ObservationError::MissingAspect {
                entity: key,
                name: "record".to_owned(),
            });
        };
        let value = decode_entity(key, state)?;
        entities.insert(key, value);
    }
    for key in program.definition().entities.keys() {
        if !entities.contains_key(key) {
            return Err(ObservationError::MissingEntity(*key));
        }
    }
    Ok(entities)
}

fn observe_relations(
    handles: &SupplyChainSemanticHandles,
    view: &RelationalReadView,
) -> Result<BTreeMap<RelationKey, RelationEdge>, ObservationError> {
    let mut relations = BTreeMap::new();
    for record in view.relations() {
        let Some(key) = handles.relation_key(record.relation_id) else {
            return Err(ObservationError::UnknownRelationIdentity(
                record.relation_id,
            ));
        };
        if record.kind.kind_id != super::program::relation_kind_id(key.kind) {
            return Err(ObservationError::WrongRelationKind {
                key,
                observed: record.kind.kind_id,
            });
        }
        let (Some(source), Some(target)) = (
            handles.entity_key(record.source),
            handles.entity_key(record.target),
        ) else {
            return Err(ObservationError::UnknownRelationEndpoint(key));
        };
        relations.insert(
            key,
            RelationEdge {
                key,
                source,
                target,
            },
        );
    }
    for key in handles.relations.keys() {
        if !relations.contains_key(key) {
            return Err(ObservationError::MissingRelation(*key));
        }
    }
    Ok(relations)
}

fn decode_entity(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
) -> Result<EntityRecord, ObservationError> {
    match key.kind {
        EntityKind::Port => Ok(EntityRecord::Port(PortRecord {
            code: u16_value(key, state, "port_code")?,
            name: string_value(key, state, "name")?,
            region: debug_region(key, state, "region")?,
            posture: debug_posture(key, state, "posture")?,
        })),
        EntityKind::Terminal => Ok(EntityRecord::Terminal(TerminalRecord {
            name: string_value(key, state, "name")?,
            capacity: super::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        EntityKind::Berth => Ok(EntityRecord::Berth(super::schema::BerthRecord {
            name: string_value(key, state, "name")?,
            depth: super::schema::DepthMeters(u16_value(key, state, "depth")?),
            capacity: super::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        EntityKind::Vessel => Ok(EntityRecord::Vessel(VesselRecord {
            call_sign: string_value(key, state, "call_sign")?,
            class: debug_class(key, state, "class")?,
            capacity: super::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        EntityKind::Voyage => Ok(EntityRecord::Voyage(VoyageRecord {
            status: debug_status(key, state, "status")?,
            departure: super::schema::Minute(u32_value(key, state, "departure")?),
            arrival: super::schema::Minute(u32_value(key, state, "arrival")?),
            revision: u16_value(key, state, "revision")?,
        })),
        EntityKind::PortCall => Ok(EntityRecord::PortCall(PortCallRecord {
            sequence: u16_value(key, state, "sequence")?,
            revision: u16_value(key, state, "revision")?,
        })),
        EntityKind::CargoLot => Ok(EntityRecord::CargoLot(CargoLotRecord {
            mass: super::schema::MassTons(u32_value(key, state, "mass")?),
            customer_code: super::schema::CargoCode(string_value(key, state, "customer_code")?),
            hazard: debug_hazard(key, state, "hazard")?,
            booking: debug_booking(key, state, "booking")?,
        })),
        EntityKind::Inspection => Ok(EntityRecord::Inspection(InspectionRecord {
            result: debug_result(key, state, "result")?,
            minute: super::schema::Minute(u32_value(key, state, "minute")?),
        })),
    }
}

fn value<'a>(
    key: EntityKey,
    state: &'a AuthoritativeRecordAspectState,
    name: &str,
) -> Result<&'a AspectValue, ObservationError> {
    let aspect = AspectKey::new(name).expect("canonical Supply Chain aspect key");
    let Some(entry) = state.get(&aspect) else {
        return Err(ObservationError::MissingAspect {
            entity: key,
            name: name.to_owned(),
        });
    };
    match entry.view() {
        ContractValidatedAspectValueView::Scalar(value) => Ok(value),
        ContractValidatedAspectValueView::Struct(_) => Err(ObservationError::InvalidAspect {
            entity: key,
            name: name.to_owned(),
            detail: "expected scalar value".to_owned(),
        }),
    }
}

pub(super) fn string_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<String, ObservationError> {
    match value(key, state, name)? {
        AspectValue::String(InternedString::Raw(value)) => Ok(value.clone()),
        _ => Err(invalid(key, name, "expected raw string")),
    }
}

fn u16_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<u16, ObservationError> {
    match value(key, state, name)? {
        AspectValue::UInt64(value) => (*value)
            .try_into()
            .map_err(|_| invalid(key, name, "UInt64 value exceeds u16")),
        _ => Err(invalid(key, name, "expected UInt64")),
    }
}

fn u32_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<u32, ObservationError> {
    match value(key, state, name)? {
        AspectValue::UInt64(value) => (*value)
            .try_into()
            .map_err(|_| invalid(key, name, "UInt64 value exceeds u32")),
        _ => Err(invalid(key, name, "expected UInt64")),
    }
}

pub(super) fn invalid(key: EntityKey, name: &str, detail: &str) -> ObservationError {
    ObservationError::InvalidAspect {
        entity: key,
        name: name.to_owned(),
        detail: detail.to_owned(),
    }
}

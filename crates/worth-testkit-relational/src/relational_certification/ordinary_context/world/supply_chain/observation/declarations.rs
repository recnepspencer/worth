pub use {std::collections::BTreeMap, std::collections::BTreeSet};

pub use {crate::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_booking, crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_class, crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_hazard, crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_posture, crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_region, crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_result, crate::relational_certification::ordinary_context::world::supply_chain::observation_debug::debug_status};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::CargoLotRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::PortCallRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::PortRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::TerminalRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::VesselRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageRecord};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey};
pub use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::AuthoritativeRecordAspectState, worth_foundational::facade::ContractValidatedAspectValueView, worth_foundational::facade::InternedString};
pub use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::KindId, worth_relational::facade::identity::RelationId};
pub use {worth_relational::facade::runtime::RelationalReadView, worth_relational::facade::runtime::RelationalRuntime};
pub use {worth_relational::facade::snapshots::SnapshotHandle};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationError {
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
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

pub fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
) -> Result<ObservedSupplyChainState, ObservationError> {
    let observation = world.basis.observation();
    if observation.identity().runtime_instance_id() != world.handles.branch.runtime_instance_id
        || observation.identity().branch_id() != &world.handles.branch.branch_id
    {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable);
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
pub fn observe_observation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    runtime: &RelationalRuntime,
    observation: &worth_relational::facade::branch::RelationalBranchObservation,
) -> Result<ObservedSupplyChainState, ObservationError> {
    if observation.identity().runtime_instance_id() != handles.branch.runtime_instance_id {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable);
    }
    let view = runtime
        .read_truth()
        .read_observation(execution, observation)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::Execution)?
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::BranchBasis)?;
    let schema = runtime
        .read_truth()
        .observation_schema_version(observation)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::BranchBasis)
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
pub fn observe_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<ObservedSupplyChainState, ObservationError> {
    if snapshot.runtime_instance_id() != handles.branch.runtime_instance_id
        || snapshot.branch_id() != &handles.branch.branch_id
    {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable);
    }
    let Some(view) = runtime
        .read_truth()
        .read_snapshot(snapshot, execution)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::Execution)?
    else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable);
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

pub fn assemble_observation(
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    runtime: &RelationalRuntime,
    branch_id: &worth_relational::facade::history::BranchId,
    schema: crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion,
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
        absent_entities: std::collections::BTreeSet::new(),
        absent_relations: std::collections::BTreeSet::new(),
        branch,
        parent,
        lineage,
        accepted: Vec::new(),
        history: Vec::new(),
    })
}

pub fn observe_schema_version(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, ObservationError> {
    let observed = runtime
        .read_truth()
        .snapshot_schema_version(snapshot)
        .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::SnapshotUnavailable)?;
    schema_version(observed)
}

pub fn schema_version(
    observed: worth_relational::facade::schema::SchemaVersionId,
) -> Result<crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, ObservationError> {
    match observed.0 {
        1 => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1),
        2 => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2),
        _ => Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnsupportedSchemaVersion(observed)),
    }
}

pub fn branch_label(
    branch_id: &worth_relational::facade::history::BranchId,
) -> Result<BranchLabel, ObservationError> {
    match branch_id.0.as_str() {
        "main" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
        "storm" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
        "maintenance" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance),
        "customs" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs),
        "medical-hold" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold),
        "southpoint-expansion" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::SouthpointExpansion),
        "competing-arrival" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival),
        "inspection" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Inspection),
        "rewire" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire),
        "hazard-v2" | "hazard-v2-secondary" => Ok(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2),
        _ => Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnknownBranch(branch_id.clone())),
    }
}

pub fn observe_entities(
    program: &CompiledSupplyChainProgram,
    handles: &SupplyChainSemanticHandles,
    view: &RelationalReadView,
) -> Result<BTreeMap<EntityKey, EntityRecord>, ObservationError> {
    let mut entities = std::collections::BTreeMap::new();
    for record in view.entities() {
        let Some(key) = handles.entity_key(record.entity_id) else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnknownEntityIdentity(record.entity_id));
        };
        let expected_kind = key.kind;
        if record.kind.kind_id != crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(expected_kind) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::WrongEntityKind {
                key,
                observed: record.kind.kind_id,
            });
        }
        let Some(state) = record.authoritative_aspect_state.as_ref() else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::MissingAspect {
                entity: key,
                name: "record".to_owned(),
            });
        };
        let value = decode_entity(key, state)?;
        entities.insert(key, value);
    }
    for key in program.definition().entities.keys() {
        if !entities.contains_key(key) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::MissingEntity(*key));
        }
    }
    Ok(entities)
}

pub fn observe_relations(
    handles: &SupplyChainSemanticHandles,
    view: &RelationalReadView,
) -> Result<BTreeMap<RelationKey, RelationEdge>, ObservationError> {
    let mut relations = std::collections::BTreeMap::new();
    for record in view.relations() {
        let Some(key) = handles.relation_key(record.relation_id) else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnknownRelationIdentity(
                record.relation_id,
            ));
        };
        if record.kind.kind_id != crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id(key.kind) {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::WrongRelationKind {
                key,
                observed: record.kind.kind_id,
            });
        }
        let (Some(source), Some(target)) = (
            handles.entity_key(record.source),
            handles.entity_key(record.target),
        ) else {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::UnknownRelationEndpoint(key));
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
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::MissingRelation(*key));
        }
    }
    Ok(relations)
}

pub fn decode_entity(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
) -> Result<EntityRecord, ObservationError> {
    match key.kind {
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(PortRecord {
            code: u16_value(key, state, "port_code")?,
            name: string_value(key, state, "name")?,
            region: debug_region(key, state, "region")?,
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(TerminalRecord {
            name: string_value(key, state, "name")?,
            capacity: crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(crate::relational_certification::ordinary_context::world::supply_chain::schema::BerthRecord {
            name: string_value(key, state, "name")?,
            depth: crate::relational_certification::ordinary_context::world::supply_chain::schema::DepthMeters(u16_value(key, state, "depth")?),
            capacity: crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Vessel(VesselRecord {
            call_sign: string_value(key, state, "call_sign")?,
            class: debug_class(key, state, "class")?,
            capacity: crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity(u32_value(key, state, "capacity")?),
            posture: debug_posture(key, state, "posture")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(VoyageRecord {
            status: debug_status(key, state, "status")?,
            departure: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(u32_value(key, state, "departure")?),
            arrival: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(u32_value(key, state, "arrival")?),
            revision: u16_value(key, state, "revision")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(PortCallRecord {
            sequence: u16_value(key, state, "sequence")?,
            revision: u16_value(key, state, "revision")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(CargoLotRecord {
            mass: crate::relational_certification::ordinary_context::world::supply_chain::schema::MassTons(u32_value(key, state, "mass")?),
            customer_code: crate::relational_certification::ordinary_context::world::supply_chain::schema::CargoCode(string_value(key, state, "customer_code")?),
            hazard: debug_hazard(key, state, "hazard")?,
            booking: debug_booking(key, state, "booking")?,
        })),
        crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => Ok(crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(InspectionRecord {
            result: debug_result(key, state, "result")?,
            minute: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(u32_value(key, state, "minute")?),
        })),
    }
}

pub fn value<'a>(
    key: EntityKey,
    state: &'a AuthoritativeRecordAspectState,
    name: &str,
) -> Result<&'a AspectValue, ObservationError> {
    let aspect = worth_foundational::facade::AspectKey::new(name).expect("canonical Supply Chain aspect key");
    let Some(entry) = state.get(&aspect) else {
        return Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::MissingAspect {
            entity: key,
            name: name.to_owned(),
        });
    };
    match entry.view() {
        worth_foundational::facade::ContractValidatedAspectValueView::Scalar(value) => Ok(value),
        worth_foundational::facade::ContractValidatedAspectValueView::Struct(_) => Err(crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::InvalidAspect {
            entity: key,
            name: name.to_owned(),
            detail: "expected scalar value".to_owned(),
        }),
    }
}

pub fn string_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<String, ObservationError> {
    match value(key, state, name)? {
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw(value)) => Ok(value.clone()),
        _ => Err(invalid(key, name, "expected raw string")),
    }
}

pub fn u16_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<u16, ObservationError> {
    match value(key, state, name)? {
        worth_foundational::facade::AspectValue::UInt64(value) => (*value)
            .try_into()
            .map_err(|_| invalid(key, name, "UInt64 value exceeds u16")),
        _ => Err(invalid(key, name, "expected UInt64")),
    }
}

pub fn u32_value(
    key: EntityKey,
    state: &AuthoritativeRecordAspectState,
    name: &str,
) -> Result<u32, ObservationError> {
    match value(key, state, name)? {
        worth_foundational::facade::AspectValue::UInt64(value) => (*value)
            .try_into()
            .map_err(|_| invalid(key, name, "UInt64 value exceeds u32")),
        _ => Err(invalid(key, name, "expected UInt64")),
    }
}

pub fn invalid(key: EntityKey, name: &str, detail: &str) -> ObservationError {
    crate::relational_certification::ordinary_context::world::supply_chain::observation::ObservationError::InvalidAspect {
        entity: key,
        name: name.to_owned(),
        detail: detail.to_owned(),
    }
}
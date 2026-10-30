pub use {crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, crate::relational_certification::ordinary_context::world::supply_chain::schema::Capacity, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture, crate::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageRecord, crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind};
pub use {crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError, crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OracleApplicationError {
    MissingEntity {
        delta: DeltaId,
        key: EntityKey,
    },
    MissingRelation {
        delta: DeltaId,
        key: RelationKey,
    },
    SiblingFactLeak {
        expected: BranchLabel,
        observed: BranchLabel,
        delta: DeltaId,
    },
    WrongAncestry(AncestryError),
    DuplicateRelation(RelationKey),
    DuplicateDelta(DeltaId),
    InvalidSchemaTransition {
        expected: crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion,
        observed: crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion,
    },
    InvalidPostState(SchemaError),
}

pub fn entity(
    branch: &OracleBranch,
    delta: DeltaId,
    key: EntityKey,
) -> Result<&EntityRecord, OracleApplicationError> {
    branch
        .state
        .entity(key)
        .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key })
}

pub fn relation(
    branch: &OracleBranch,
    delta: DeltaId,
    key: RelationKey,
) -> Result<&RelationEdge, OracleApplicationError> {
    branch
        .state
        .relation(key)
        .ok_or(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingRelation { delta, key })
}

pub fn check_preconditions(
    branch: &OracleBranch,
    delta: DeltaId,
) -> Result<(), OracleApplicationError> {
    let contract = delta.contract();
    for precondition in contract.preconditions {
        match precondition {
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(key) => {
                entity(branch, delta, key)?;
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(key) => {
                relation(branch, delta, key)?;
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(expected) => {
                let observed = branch.state.schema_version();
                if observed != expected {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::InvalidSchemaTransition {
                        expected,
                        observed,
                    });
                }
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(expected) => {
                if branch.ancestry.branch != expected {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::SiblingFactLeak {
                        expected,
                        observed: branch.ancestry.branch,
                        delta,
                    });
                }
            }
            crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(expected) => {
                if branch
                    .ancestry
                    .history
                    .iter()
                    .any(|event| event.delta == expected)
                {
                    return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::DuplicateDelta(expected));
                }
            }
        }
    }
    Ok(())
}

pub fn apply(
    branch: &OracleBranch,
    delta: DeltaId,
) -> Result<OracleBranch, OracleApplicationError> {
    check_preconditions(branch, delta)?;
    let mut next = branch.clone();
    let voyage = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0);
    let cargo = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0);
    let atlas = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 0);
    let call = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 1);
    let relation_key = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    match delta {
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value) = entity(branch, delta, voyage)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: voyage });
            };
            let VoyageRecord {
                status: _,
                departure,
                arrival,
                revision,
                ..
            } = value;
            next.state = next.state.replace_entity(
                voyage,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(VoyageRecord {
                    status: crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Rerouted,
                    departure,
                    arrival: crate::relational_certification::ordinary_context::world::supply_chain::schema::Minute(arrival.0 + 30),
                    revision: revision + 1,
                }),
            );
            let edge = relation(branch, delta, relation_key)?;
            next.state = next.state.replace_relation(RelationEdge {
                key: edge.key,
                source: edge.source,
                target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2),
            });
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(mut berth) = entity(branch, delta, atlas)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: atlas });
            };
            berth.posture = crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Maintenance;
            next.state = next.state.replace_entity(atlas, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(berth));
            let key = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0);
            let edge = relation(branch, delta, key)?;
            next.state = next.state.replace_relation(RelationEdge {
                key: edge.key,
                source: edge.source,
                target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 1),
            });
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(mut value) = entity(branch, delta, voyage)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: voyage });
            };
            value.status = crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Delayed;
            value.arrival.0 += 60;
            value.revision += 1;
            next.state = next
                .state
                .replace_entity(voyage, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(mut lot) = entity(branch, delta, cargo)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: cargo });
            };
            lot.booking = crate::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Held;
            next.state = next
                .state
                .replace_entity(cargo, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(lot));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => {
            let terminal = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal, 1);
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(mut value) = entity(branch, delta, terminal)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity {
                    delta,
                    key: terminal,
                });
            };
            value.capacity = Capacity(value.capacity.0 + 1_000);
            next.state = next
                .state
                .replace_entity(terminal, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Terminal(value));
            let berth = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 2);
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(mut value) = entity(branch, delta, berth)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: berth });
            };
            value.capacity.0 += 50;
            next.state = next.state.replace_entity(berth, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(value));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(mut value) = entity(branch, delta, voyage)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: voyage });
            };
            value.arrival.0 += 50;
            value.revision += 1;
            value.status = crate::relational_certification::ordinary_context::world::supply_chain::schema::VoyageStatus::Delayed;
            next.state = next
                .state
                .replace_entity(voyage, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(value));
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(mut berth) = entity(branch, delta, atlas)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: atlas });
            };
            berth.posture = crate::relational_certification::ordinary_context::world::supply_chain::schema::OperatingPosture::Retired;
            next.state = next.state.replace_entity(atlas, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Berth(berth));
            let inspection = crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection, 0);
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(InspectionRecord { minute, .. }) =
                entity(branch, delta, inspection)?.clone()
            else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity {
                    delta,
                    key: inspection,
                });
            };
            next.state = next.state.replace_entity(
                inspection,
                crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Inspection(InspectionRecord {
                    result: crate::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult::Flagged,
                    minute,
                }),
            );
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(mut value) = entity(branch, delta, call)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: call });
            };
            value.revision += 1;
            next.state = next
                .state
                .replace_entity(call, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(value));
            let edge = relation(branch, delta, relation_key)?;
            next.state = next.state.replace_relation(RelationEdge {
                key: edge.key,
                source: edge.source,
                target: crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 3),
            });
        }
        crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => {
            let crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(mut lot) = entity(branch, delta, cargo)?.clone() else {
                return Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity { delta, key: cargo });
            };
            lot.hazard = crate::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass::HazardousV2;
            next.state = next.state.upgrade_hazard_schema();
            next.state = next
                .state
                .replace_entity(cargo, crate::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(lot));
        }
    }
    next.state
        .validate_complete()
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::InvalidPostState)?;
    Ok(next.accept(delta))
}

pub fn apply_from_parent(
    parent: &OracleBranch,
    child: &OracleBranch,
    delta: DeltaId,
) -> Result<OracleBranch, OracleApplicationError> {
    child
        .expects_parent_ancestry(&parent.ancestry)
        .map_err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry)?;
    apply(child, delta)
}

pub fn reject_duplicate_relation(
    branch: &OracleBranch,
    key: RelationKey,
) -> Result<(), OracleApplicationError> {
    if branch.state.relation(key).is_some() {
        Err(crate::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::DuplicateRelation(key))
    } else {
        Ok(())
    }
}

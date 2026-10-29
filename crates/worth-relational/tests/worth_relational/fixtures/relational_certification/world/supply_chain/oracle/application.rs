pub(crate) use super::super::delta::{DeltaId, DeltaPrecondition};
pub(crate) use super::super::schema::{
    BookingStatus, Capacity, EntityRecord, HazardClass, InspectionRecord, InspectionResult,
    OperatingPosture, RelationEdge, SchemaError, VoyageRecord, VoyageStatus,
};
pub(crate) use super::super::semantic_key::{
    BranchLabel, EntityKey, EntityKind, RelationKey, RelationKind,
};
pub(crate) use super::ancestry::{AncestryError, OracleBranch};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum OracleApplicationError {
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
        expected: super::super::schema::SchemaVersion,
        observed: super::super::schema::SchemaVersion,
    },
    InvalidPostState(SchemaError),
}

pub(crate) fn entity(
    branch: &OracleBranch,
    delta: DeltaId,
    key: EntityKey,
) -> Result<&EntityRecord, OracleApplicationError> {
    branch
        .state
        .entity(key)
        .ok_or(OracleApplicationError::MissingEntity { delta, key })
}

pub(crate) fn relation(
    branch: &OracleBranch,
    delta: DeltaId,
    key: RelationKey,
) -> Result<&RelationEdge, OracleApplicationError> {
    branch
        .state
        .relation(key)
        .ok_or(OracleApplicationError::MissingRelation { delta, key })
}

pub(crate) fn check_preconditions(
    branch: &OracleBranch,
    delta: DeltaId,
) -> Result<(), OracleApplicationError> {
    let contract = delta.contract();
    for precondition in contract.preconditions {
        match precondition {
            DeltaPrecondition::EntityPresent(key) => {
                entity(branch, delta, key)?;
            }
            DeltaPrecondition::RelationPresent(key) => {
                relation(branch, delta, key)?;
            }
            DeltaPrecondition::Schema(expected) => {
                let observed = branch.state.schema_version();
                if observed != expected {
                    return Err(OracleApplicationError::InvalidSchemaTransition {
                        expected,
                        observed,
                    });
                }
            }
            DeltaPrecondition::Branch(expected) => {
                if branch.ancestry.branch != expected {
                    return Err(OracleApplicationError::SiblingFactLeak {
                        expected,
                        observed: branch.ancestry.branch,
                        delta,
                    });
                }
            }
            DeltaPrecondition::DeltaNotAccepted(expected) => {
                if branch
                    .ancestry
                    .history
                    .iter()
                    .any(|event| event.delta == expected)
                {
                    return Err(OracleApplicationError::DuplicateDelta(expected));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn apply(
    branch: &OracleBranch,
    delta: DeltaId,
) -> Result<OracleBranch, OracleApplicationError> {
    check_preconditions(branch, delta)?;
    let mut next = branch.clone();
    let voyage = EntityKey::new(EntityKind::Voyage, 0);
    let cargo = EntityKey::new(EntityKind::CargoLot, 0);
    let atlas = EntityKey::new(EntityKind::Berth, 0);
    let call = EntityKey::new(EntityKind::PortCall, 1);
    let relation_key = RelationKey::new(RelationKind::CallAtPort, 1);
    match delta {
        DeltaId::StormRerouteAurora => {
            let EntityRecord::Voyage(value) = entity(branch, delta, voyage)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: voyage });
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
                EntityRecord::Voyage(VoyageRecord {
                    status: VoyageStatus::Rerouted,
                    departure,
                    arrival: super::super::schema::Minute(arrival.0 + 30),
                    revision: revision + 1,
                }),
            );
            let edge = relation(branch, delta, relation_key)?;
            next.state = next.state.replace_relation(RelationEdge {
                key: edge.key,
                source: edge.source,
                target: EntityKey::new(EntityKind::Port, 2),
            });
        }
        DeltaId::MaintainAtlasBerth => {
            let EntityRecord::Berth(mut berth) = entity(branch, delta, atlas)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: atlas });
            };
            berth.posture = OperatingPosture::Maintenance;
            next.state = next.state.replace_entity(atlas, EntityRecord::Berth(berth));
            let key = RelationKey::new(RelationKind::VesselAssignedToBerth, 0);
            let edge = relation(branch, delta, key)?;
            next.state = next.state.replace_relation(RelationEdge {
                key: edge.key,
                source: edge.source,
                target: EntityKey::new(EntityKind::Berth, 1),
            });
            let EntityRecord::Voyage(mut value) = entity(branch, delta, voyage)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: voyage });
            };
            value.status = VoyageStatus::Delayed;
            value.arrival.0 += 60;
            value.revision += 1;
            next.state = next
                .state
                .replace_entity(voyage, EntityRecord::Voyage(value));
        }
        DeltaId::HoldMedicalCargo => {
            let EntityRecord::CargoLot(mut lot) = entity(branch, delta, cargo)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: cargo });
            };
            lot.booking = BookingStatus::Held;
            next.state = next
                .state
                .replace_entity(cargo, EntityRecord::CargoLot(lot));
        }
        DeltaId::ExpandSouthpointCapacity => {
            let terminal = EntityKey::new(EntityKind::Terminal, 1);
            let EntityRecord::Terminal(mut value) = entity(branch, delta, terminal)?.clone() else {
                return Err(OracleApplicationError::MissingEntity {
                    delta,
                    key: terminal,
                });
            };
            value.capacity = Capacity(value.capacity.0 + 1_000);
            next.state = next
                .state
                .replace_entity(terminal, EntityRecord::Terminal(value));
            let berth = EntityKey::new(EntityKind::Berth, 2);
            let EntityRecord::Berth(mut value) = entity(branch, delta, berth)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: berth });
            };
            value.capacity.0 += 50;
            next.state = next.state.replace_entity(berth, EntityRecord::Berth(value));
        }
        DeltaId::CompetingAuroraArrival => {
            let EntityRecord::Voyage(mut value) = entity(branch, delta, voyage)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: voyage });
            };
            value.arrival.0 += 50;
            value.revision += 1;
            value.status = VoyageStatus::Delayed;
            next.state = next
                .state
                .replace_entity(voyage, EntityRecord::Voyage(value));
        }
        DeltaId::RetireAtlasWhileInspectingAurora => {
            let EntityRecord::Berth(mut berth) = entity(branch, delta, atlas)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: atlas });
            };
            berth.posture = OperatingPosture::Retired;
            next.state = next.state.replace_entity(atlas, EntityRecord::Berth(berth));
            let inspection = EntityKey::new(EntityKind::Inspection, 0);
            let EntityRecord::Inspection(InspectionRecord { minute, .. }) =
                entity(branch, delta, inspection)?.clone()
            else {
                return Err(OracleApplicationError::MissingEntity {
                    delta,
                    key: inspection,
                });
            };
            next.state = next.state.replace_entity(
                inspection,
                EntityRecord::Inspection(InspectionRecord {
                    result: InspectionResult::Flagged,
                    minute,
                }),
            );
        }
        DeltaId::RewireAuroraPortCall => {
            let EntityRecord::PortCall(mut value) = entity(branch, delta, call)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: call });
            };
            value.revision += 1;
            next.state = next
                .state
                .replace_entity(call, EntityRecord::PortCall(value));
            let edge = relation(branch, delta, relation_key)?;
            next.state = next.state.replace_relation(RelationEdge {
                key: edge.key,
                source: edge.source,
                target: EntityKey::new(EntityKind::Port, 3),
            });
        }
        DeltaId::AdoptHazardClassificationV2 => {
            let EntityRecord::CargoLot(mut lot) = entity(branch, delta, cargo)?.clone() else {
                return Err(OracleApplicationError::MissingEntity { delta, key: cargo });
            };
            lot.hazard = HazardClass::HazardousV2;
            next.state = next.state.upgrade_hazard_schema();
            next.state = next
                .state
                .replace_entity(cargo, EntityRecord::CargoLot(lot));
        }
    }
    next.state
        .validate_complete()
        .map_err(OracleApplicationError::InvalidPostState)?;
    Ok(next.accept(delta))
}

pub(crate) fn apply_from_parent(
    parent: &OracleBranch,
    child: &OracleBranch,
    delta: DeltaId,
) -> Result<OracleBranch, OracleApplicationError> {
    child
        .expects_parent_ancestry(&parent.ancestry)
        .map_err(OracleApplicationError::WrongAncestry)?;
    apply(child, delta)
}

pub(crate) fn reject_duplicate_relation(
    branch: &OracleBranch,
    key: RelationKey,
) -> Result<(), OracleApplicationError> {
    if branch.state.relation(key).is_some() {
        Err(OracleApplicationError::DuplicateRelation(key))
    } else {
        Ok(())
    }
}

pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::read_footprint::DeltaReadFootprint, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

pub(crate) fn expected_read(id: DeltaId) -> DeltaReadFootprint {
    let voyage = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
    let cargo = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity();
    let atlas = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity();
    let beacon = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Beacon.entity();
    let terminal = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointContainer.entity();
    let berth = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth.entity();
    let inspection = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival.entity();
    let call = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity();
    let call_relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let assignment = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0);
    let port_one = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 1);
    let port_two = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2);
    let port_three = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 3);
    let (entities, relations, fields, branch, schema) = match id {
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => (
            [voyage, port_one, port_two].into_iter().collect(),
            [call_relation].into_iter().collect(),
            fields(&[
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
            ]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => (
            [atlas, beacon, voyage].into_iter().collect(),
            [assignment].into_iter().collect(),
            fields(&[
                (atlas, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
            ]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => (
            [cargo].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[(cargo, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::BookingStatus)]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => (
            [terminal, berth].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[(terminal, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity), (berth, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity)]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::SouthpointExpansion,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => (
            [voyage].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
            ]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => (
            [atlas, inspection].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[
                (atlas, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture),
                (inspection, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionResult),
                (inspection, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionMinute),
            ]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Inspection,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => (
            [call, port_one, port_three].into_iter().collect(),
            [call_relation].into_iter().collect(),
            fields(&[(call, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => (
            [cargo].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[(cargo, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::HazardClass)]),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
        ),
    };
    DeltaReadFootprint {
        entities,
        relations,
        fields,
        schema: Some(schema),
        branch: Some(branch),
        history: Some(id),
    }
}

pub(crate) fn fields(values: &[(EntityKey, FieldKey)]) -> BTreeSet<SemanticPath> {
    values
        .iter()
        .map(|(entity, field)| worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(*entity, *field))
        .collect()
}

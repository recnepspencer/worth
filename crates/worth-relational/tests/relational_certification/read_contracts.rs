use std::collections::BTreeSet;

use crate::relational_certification::world::supply_chain::*;

#[test]
fn every_delta_has_an_independent_hand_authored_read_footprint() {
    for id in DeltaId::ALL {
        assert_eq!(
            id.contract().read,
            expected_read(id),
            "read contract for {id:?}"
        );
    }
}

#[test]
fn read_footprint_mutations_are_detected_on_each_history_and_entity_axis() {
    let contract = DeltaId::StormRerouteAurora.contract();
    let mut missing_entity = expected_read(DeltaId::StormRerouteAurora);
    missing_entity
        .entities
        .remove(&EntityKey::new(EntityKind::Port, 2));
    assert_ne!(missing_entity, contract.read);

    let mut missing_relation = expected_read(DeltaId::StormRerouteAurora);
    missing_relation
        .relations
        .remove(&RelationKey::new(RelationKind::CallAtPort, 1));
    assert_ne!(missing_relation, contract.read);

    let mut missing_field = expected_read(DeltaId::StormRerouteAurora);
    missing_field.fields.remove(&SemanticPath::field(
        Anchor::AuroraEastbound.entity(),
        FieldKey::Revision,
    ));
    assert_ne!(missing_field, contract.read);

    let mut wrong_history = expected_read(DeltaId::StormRerouteAurora);
    wrong_history.history = Some(DeltaId::CompetingAuroraArrival);
    assert_ne!(wrong_history, contract.read);
}

pub(crate) fn expected_read(id: DeltaId) -> DeltaReadFootprint {
    let voyage = Anchor::AuroraEastbound.entity();
    let cargo = Anchor::MedicalSupplies.entity();
    let atlas = Anchor::Atlas.entity();
    let beacon = Anchor::Beacon.entity();
    let terminal = Anchor::SouthpointContainer.entity();
    let berth = Anchor::SouthpointBerth.entity();
    let inspection = Anchor::AuroraArrival.entity();
    let call = Anchor::AuroraSouthpoint.entity();
    let call_relation = RelationKey::new(RelationKind::CallAtPort, 1);
    let assignment = RelationKey::new(RelationKind::VesselAssignedToBerth, 0);
    let port_one = EntityKey::new(EntityKind::Port, 1);
    let port_two = EntityKey::new(EntityKind::Port, 2);
    let port_three = EntityKey::new(EntityKind::Port, 3);
    let (entities, relations, fields, branch, schema) = match id {
        DeltaId::StormRerouteAurora => (
            [voyage, port_one, port_two].into_iter().collect(),
            [call_relation].into_iter().collect(),
            fields(&[
                (voyage, FieldKey::Status),
                (voyage, FieldKey::ArrivalMinute),
                (voyage, FieldKey::Revision),
            ]),
            BranchLabel::Storm,
            SchemaVersion::V1,
        ),
        DeltaId::MaintainAtlasBerth => (
            [atlas, beacon, voyage].into_iter().collect(),
            [assignment].into_iter().collect(),
            fields(&[
                (atlas, FieldKey::Posture),
                (voyage, FieldKey::Status),
                (voyage, FieldKey::ArrivalMinute),
                (voyage, FieldKey::Revision),
            ]),
            BranchLabel::Maintenance,
            SchemaVersion::V1,
        ),
        DeltaId::HoldMedicalCargo => (
            [cargo].into_iter().collect(),
            BTreeSet::new(),
            fields(&[(cargo, FieldKey::BookingStatus)]),
            BranchLabel::MedicalHold,
            SchemaVersion::V1,
        ),
        DeltaId::ExpandSouthpointCapacity => (
            [terminal, berth].into_iter().collect(),
            BTreeSet::new(),
            fields(&[(terminal, FieldKey::Capacity), (berth, FieldKey::Capacity)]),
            BranchLabel::SouthpointExpansion,
            SchemaVersion::V1,
        ),
        DeltaId::CompetingAuroraArrival => (
            [voyage].into_iter().collect(),
            BTreeSet::new(),
            fields(&[
                (voyage, FieldKey::Status),
                (voyage, FieldKey::ArrivalMinute),
                (voyage, FieldKey::Revision),
            ]),
            BranchLabel::CompetingArrival,
            SchemaVersion::V1,
        ),
        DeltaId::RetireAtlasWhileInspectingAurora => (
            [atlas, inspection].into_iter().collect(),
            BTreeSet::new(),
            fields(&[
                (atlas, FieldKey::Posture),
                (inspection, FieldKey::InspectionResult),
                (inspection, FieldKey::InspectionMinute),
            ]),
            BranchLabel::Inspection,
            SchemaVersion::V1,
        ),
        DeltaId::RewireAuroraPortCall => (
            [call, port_one, port_three].into_iter().collect(),
            [call_relation].into_iter().collect(),
            fields(&[(call, FieldKey::Revision)]),
            BranchLabel::Rewire,
            SchemaVersion::V1,
        ),
        DeltaId::AdoptHazardClassificationV2 => (
            [cargo].into_iter().collect(),
            BTreeSet::new(),
            fields(&[(cargo, FieldKey::HazardClass)]),
            BranchLabel::HazardV2,
            SchemaVersion::V1,
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

fn fields(values: &[(EntityKey, FieldKey)]) -> BTreeSet<SemanticPath> {
    values
        .iter()
        .map(|(entity, field)| SemanticPath::field(*entity, *field))
        .collect()
}

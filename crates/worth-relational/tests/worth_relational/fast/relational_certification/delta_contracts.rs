pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use crate::fast::relational_certification::field_values::field_value;
pub(crate) use crate::fast::relational_certification::world::supply_chain::*;

#[test]
fn rewiring_preserves_relation_identity_and_hazard_v2_is_schema_typed() {
    let rewire = DeltaId::RewireAuroraPortCall.contract();
    let call_relation = RelationKey::new(RelationKind::CallAtPort, 1);
    assert!(rewire.footprint.relations.contains(&call_relation));
    assert!(rewire
        .postconditions
        .contains(&DeltaPostcondition::RelationTargetChanges(call_relation)));
    assert!(rewire
        .postconditions
        .contains(&DeltaPostcondition::RelationSourceUnchanged(call_relation)));
    assert!(rewire
        .identity_basis
        .contains(&DeltaIdentityBasis::Relation(call_relation)));

    let hazard = DeltaId::AdoptHazardClassificationV2.contract();
    assert_eq!(hazard.schema_meaning, SchemaVersion::V2);
    assert!(hazard
        .postconditions
        .contains(&DeltaPostcondition::SchemaChanges(SchemaVersion::V2)));
}

#[derive(Debug)]
pub(crate) struct ExpectedEffect {
    pub(crate) entities: BTreeSet<EntityKey>,
    pub(crate) relations: BTreeSet<RelationKey>,
    pub(crate) relation_edges: BTreeMap<RelationKey, RelationEdge>,
    pub(crate) fields: BTreeSet<SemanticPath>,
    pub(crate) schema: Option<SchemaVersion>,
}

pub(crate) fn expected_effect(id: DeltaId, before: &OracleState) -> ExpectedEffect {
    let voyage = Anchor::AuroraEastbound.entity();
    let cargo = Anchor::MedicalSupplies.entity();
    let atlas = Anchor::Atlas.entity();
    let southpoint_terminal = Anchor::SouthpointContainer.entity();
    let southpoint_berth = Anchor::SouthpointBerth.entity();
    let inspection = Anchor::AuroraArrival.entity();
    let call = Anchor::AuroraSouthpoint.entity();
    let relation = RelationKey::new(RelationKind::CallAtPort, 1);
    let assignment = RelationKey::new(RelationKind::VesselAssignedToBerth, 0);
    let (entities, relations, fields, schema, targets) = match id {
        DeltaId::StormRerouteAurora => (
            [voyage].into_iter().collect(),
            [relation].into_iter().collect(),
            fields(&[
                (voyage, FieldKey::Status),
                (voyage, FieldKey::ArrivalMinute),
                (voyage, FieldKey::Revision),
            ]),
            None,
            vec![(relation, EntityKey::new(EntityKind::Port, 2))],
        ),
        DeltaId::MaintainAtlasBerth => (
            [atlas, voyage].into_iter().collect(),
            [assignment].into_iter().collect(),
            fields(&[
                (atlas, FieldKey::Posture),
                (voyage, FieldKey::Status),
                (voyage, FieldKey::ArrivalMinute),
                (voyage, FieldKey::Revision),
            ]),
            None,
            vec![(assignment, EntityKey::new(EntityKind::Berth, 1))],
        ),
        DeltaId::HoldMedicalCargo => (
            [cargo].into_iter().collect(),
            BTreeSet::new(),
            fields(&[(cargo, FieldKey::BookingStatus)]),
            None,
            Vec::new(),
        ),
        DeltaId::ExpandSouthpointCapacity => (
            [southpoint_terminal, southpoint_berth]
                .into_iter()
                .collect(),
            BTreeSet::new(),
            fields(&[
                (southpoint_terminal, FieldKey::Capacity),
                (southpoint_berth, FieldKey::Capacity),
            ]),
            None,
            Vec::new(),
        ),
        DeltaId::CompetingAuroraArrival => (
            [voyage].into_iter().collect(),
            BTreeSet::new(),
            fields(&[
                (voyage, FieldKey::Status),
                (voyage, FieldKey::ArrivalMinute),
                (voyage, FieldKey::Revision),
            ]),
            None,
            Vec::new(),
        ),
        DeltaId::RetireAtlasWhileInspectingAurora => (
            [atlas, inspection].into_iter().collect(),
            BTreeSet::new(),
            fields(&[
                (atlas, FieldKey::Posture),
                (inspection, FieldKey::InspectionResult),
            ]),
            None,
            Vec::new(),
        ),
        DeltaId::RewireAuroraPortCall => (
            [call].into_iter().collect(),
            [relation].into_iter().collect(),
            fields(&[(call, FieldKey::Revision)]),
            None,
            vec![(relation, EntityKey::new(EntityKind::Port, 3))],
        ),
        DeltaId::AdoptHazardClassificationV2 => (
            [cargo].into_iter().collect(),
            BTreeSet::new(),
            fields(&[(cargo, FieldKey::HazardClass)]),
            Some(SchemaVersion::V2),
            Vec::new(),
        ),
    };
    let relation_edges = targets
        .into_iter()
        .map(|(key, target)| {
            let before_edge = before.relation(key).copied().unwrap();
            (
                key,
                RelationEdge {
                    key,
                    source: before_edge.source,
                    target,
                },
            )
        })
        .collect();
    ExpectedEffect {
        entities,
        relations,
        relation_edges,
        fields,
        schema,
    }
}

pub(crate) fn fields(values: &[(EntityKey, FieldKey)]) -> BTreeSet<SemanticPath> {
    values
        .iter()
        .map(|(entity, field)| SemanticPath::field(*entity, *field))
        .collect()
}

pub(crate) fn changed_entities(before: &OracleState, after: &OracleState) -> BTreeSet<EntityKey> {
    before
        .entities
        .keys()
        .chain(after.entities.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| before.entities.get(key) != after.entities.get(key))
        .collect()
}

pub(crate) fn changed_relations(
    before: &OracleState,
    after: &OracleState,
) -> BTreeSet<RelationKey> {
    before
        .relations
        .keys()
        .chain(after.relations.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| before.relations.get(key) != after.relations.get(key))
        .collect()
}

pub(crate) fn changed_schema(before: &OracleState, after: &OracleState) -> Option<SchemaVersion> {
    (before.schema != after.schema).then_some(after.schema.version)
}

pub(crate) fn changed_fields(before: &OracleState, after: &OracleState) -> BTreeSet<SemanticPath> {
    let mut changed = BTreeSet::new();
    for key in before
        .entities
        .keys()
        .chain(after.entities.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        let (Some(before_record), Some(after_record)) =
            (before.entities.get(&key), after.entities.get(&key))
        else {
            continue;
        };
        for field in FieldKey::ALL {
            if field == FieldKey::SchemaMeaning {
                continue;
            }
            if field_value(before_record, field) != field_value(after_record, field) {
                changed.insert(SemanticPath::field(key, field));
            }
        }
    }
    changed
}

pub(crate) fn assert_all_unlisted_fields_unchanged(
    before: &OracleState,
    after: &OracleState,
    changed: &BTreeSet<SemanticPath>,
) {
    for key in before.entities.keys() {
        for field in FieldKey::ALL {
            let path = SemanticPath::field(*key, field);
            if !changed.contains(&path) {
                assert_eq!(
                    field_value(before.entities.get(key).unwrap(), field),
                    field_value(after.entities.get(key).unwrap(), field),
                    "unexpected write at {path:?}"
                );
            }
        }
    }
}

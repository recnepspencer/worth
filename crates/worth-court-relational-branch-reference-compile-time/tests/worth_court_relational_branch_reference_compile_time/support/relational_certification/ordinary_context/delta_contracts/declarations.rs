pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};

pub(crate) use {crate::relational_certification::ordinary_context::field_values::field_value};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaIdentityBasis, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::HazardClass, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::InspectionResult, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

#[derive(Debug)]
pub(crate) struct ExpectedEffect {
    pub(crate) entities: BTreeSet<EntityKey>,
    pub(crate) relations: BTreeSet<RelationKey>,
    pub(crate) relation_edges: BTreeMap<RelationKey, RelationEdge>,
    pub(crate) fields: BTreeSet<SemanticPath>,
    pub(crate) schema: Option<SchemaVersion>,
}

pub(crate) fn expected_effect(id: DeltaId, before: &OracleState) -> ExpectedEffect {
    let voyage = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity();
    let cargo = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies.entity();
    let atlas = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity();
    let southpoint_terminal = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointContainer.entity();
    let southpoint_berth = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth.entity();
    let inspection = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival.entity();
    let call = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint.entity();
    let relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let assignment = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 0);
    let (entities, relations, fields, schema, targets) = match id {
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => (
            [voyage].into_iter().collect(),
            [relation].into_iter().collect(),
            fields(&[
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
            ]),
            None,
            vec![(relation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 2))],
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => (
            [atlas, voyage].into_iter().collect(),
            [assignment].into_iter().collect(),
            fields(&[
                (atlas, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
            ]),
            None,
            vec![(assignment, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth, 1))],
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => (
            [cargo].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[(cargo, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::BookingStatus)]),
            None,
            Vec::new(),
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => (
            [southpoint_terminal, southpoint_berth]
                .into_iter()
                .collect(),
            std::collections::BTreeSet::new(),
            fields(&[
                (southpoint_terminal, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity),
                (southpoint_berth, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Capacity),
            ]),
            None,
            Vec::new(),
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => (
            [voyage].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Status),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ArrivalMinute),
                (voyage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision),
            ]),
            None,
            Vec::new(),
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => (
            [atlas, inspection].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[
                (atlas, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture),
                (inspection, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::InspectionResult),
            ]),
            None,
            Vec::new(),
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => (
            [call].into_iter().collect(),
            [relation].into_iter().collect(),
            fields(&[(call, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Revision)]),
            None,
            vec![(relation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 3))],
        ),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => (
            [cargo].into_iter().collect(),
            std::collections::BTreeSet::new(),
            fields(&[(cargo, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::HazardClass)]),
            Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2),
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
        .map(|(entity, field)| worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(*entity, *field))
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
    let mut changed = std::collections::BTreeSet::new();
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
        for field in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ALL {
            if field == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::SchemaMeaning {
                continue;
            }
            if field_value(before_record, field) != field_value(after_record, field) {
                changed.insert(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(key, field));
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
        for field in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::ALL {
            let path = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::SemanticPath::field(*key, field);
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

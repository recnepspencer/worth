use std::collections::{BTreeMap, BTreeSet};

use crate::relational_certification::field_values::field_value;
use crate::relational_certification::world::supply_chain::*;

#[test]
fn every_named_delta_has_typed_contract_and_nonempty_footprint() {
    let semantic_delta: SupplyChainScenarioDelta = DeltaId::StormRerouteAurora;
    assert_eq!(semantic_delta, DeltaId::StormRerouteAurora);
    for id in DeltaId::ALL {
        let contract = id.contract();
        assert_eq!(contract.id, id);
        assert!(!contract.footprint.entities.is_empty());
        assert!(!contract.preconditions.is_empty() || id == DeltaId::AdoptHazardClassificationV2);
        assert!(!contract.postconditions.is_empty());
        assert_eq!(contract.schema_meaning.hazard, id.hazard_schema());
        assert!(contract
            .preconditions
            .contains(&DeltaPrecondition::Schema(SchemaVersion::V1)));
        assert!(contract
            .preconditions
            .contains(&DeltaPrecondition::DeltaNotAccepted(id)));
        for precondition in &contract.preconditions {
            match precondition {
                DeltaPrecondition::EntityPresent(key) => {
                    assert!(contract.read.entities.contains(key));
                }
                DeltaPrecondition::RelationPresent(key) => {
                    assert!(contract.read.relations.contains(key));
                }
                DeltaPrecondition::Schema(_)
                | DeltaPrecondition::Branch(_)
                | DeltaPrecondition::DeltaNotAccepted(_) => {}
            }
        }
    }
}

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

#[test]
fn every_delta_has_an_independent_exact_effect_proof() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    for id in DeltaId::ALL {
        let child = baseline
            .branch
            .fork(id.branch(), BranchLabel::Operating)
            .unwrap();
        let before = child.state.clone();
        let before_ancestry = child.ancestry.clone();
        let after_branch = apply(&child, id).unwrap();
        let after = after_branch.state.clone();
        let contract = id.contract();
        let expected = expected_effect(id, &before);

        assert_eq!(changed_entities(&before, &after), expected.entities);
        assert_eq!(
            changed_entities(&before, &after),
            contract.footprint.entities
        );
        assert_eq!(changed_relations(&before, &after), expected.relations);
        assert_eq!(
            changed_relations(&before, &after),
            contract.footprint.relations
        );
        assert_eq!(changed_fields(&before, &after), expected.fields);
        assert_eq!(changed_fields(&before, &after), contract.footprint.fields);
        assert_eq!(changed_schema(&before, &after), expected.schema);
        assert_eq!(before.absent_entities, after.absent_entities);
        assert_eq!(before.absent_relations, after.absent_relations);
        assert_eq!(after_branch.ancestry.branch, before_ancestry.branch);
        assert_eq!(after_branch.ancestry.parent, before_ancestry.parent);
        assert_eq!(
            after_branch.ancestry.accepted,
            before_ancestry
                .accepted
                .iter()
                .copied()
                .chain([id])
                .collect::<Vec<_>>()
        );

        for (key, expected_edge) in expected.relation_edges {
            assert_eq!(after.relations.get(&key), Some(&expected_edge));
            assert!(contract
                .postconditions
                .contains(&DeltaPostcondition::RelationTargetChanges(key)));
            assert!(contract
                .postconditions
                .contains(&DeltaPostcondition::RelationSourceUnchanged(key)));
        }
        for path in &expected.fields {
            assert!(contract.footprint.fields.contains(path));
            let covered = contract
                .postconditions
                .contains(&DeltaPostcondition::FieldChanges(*path))
                || (path.field == Some(FieldKey::Posture)
                    && contract
                        .postconditions
                        .contains(&DeltaPostcondition::EntityPostureChanges(path.entity)));
            assert!(covered, "{id:?} omitted postcondition {path:?}");
        }
        if let Some(schema) = expected.schema {
            assert!(contract
                .postconditions
                .contains(&DeltaPostcondition::SchemaChanges(schema)));
        }
        assert_all_unlisted_fields_unchanged(&before, &after, &expected.fields);
    }
}

#[derive(Debug)]
struct ExpectedEffect {
    entities: BTreeSet<EntityKey>,
    relations: BTreeSet<RelationKey>,
    relation_edges: BTreeMap<RelationKey, RelationEdge>,
    fields: BTreeSet<SemanticPath>,
    schema: Option<SchemaVersion>,
}

fn expected_effect(id: DeltaId, before: &OracleState) -> ExpectedEffect {
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

fn fields(values: &[(EntityKey, FieldKey)]) -> BTreeSet<SemanticPath> {
    values
        .iter()
        .map(|(entity, field)| SemanticPath::field(*entity, *field))
        .collect()
}

fn changed_entities(before: &OracleState, after: &OracleState) -> BTreeSet<EntityKey> {
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

fn changed_relations(before: &OracleState, after: &OracleState) -> BTreeSet<RelationKey> {
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

fn changed_schema(before: &OracleState, after: &OracleState) -> Option<SchemaVersion> {
    (before.schema != after.schema).then_some(after.schema.version)
}

fn changed_fields(before: &OracleState, after: &OracleState) -> BTreeSet<SemanticPath> {
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

fn assert_all_unlisted_fields_unchanged(
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

#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::delta_contracts::*;

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

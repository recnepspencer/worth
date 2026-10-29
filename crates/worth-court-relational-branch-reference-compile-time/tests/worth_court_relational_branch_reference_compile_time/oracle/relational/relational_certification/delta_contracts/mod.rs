#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::SupplyChainScenarioDelta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, crate::relational_certification::ordinary_context::delta_contracts::assert_all_unlisted_fields_unchanged, crate::relational_certification::ordinary_context::delta_contracts::changed_entities, crate::relational_certification::ordinary_context::delta_contracts::changed_fields, crate::relational_certification::ordinary_context::delta_contracts::changed_relations, crate::relational_certification::ordinary_context::delta_contracts::changed_schema, crate::relational_certification::ordinary_context::delta_contracts::expected_effect, crate::relational_certification::ordinary_context::delta_contracts::fields, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

#[test]
fn every_named_delta_has_typed_contract_and_nonempty_footprint() {
    let semantic_delta: SupplyChainScenarioDelta = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora;
    assert_eq!(semantic_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora);
    for id in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ALL {
        let contract = id.contract();
        assert_eq!(contract.id, id);
        assert!(!contract.footprint.entities.is_empty());
        assert!(!contract.preconditions.is_empty() || id == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2);
        assert!(!contract.postconditions.is_empty());
        assert_eq!(contract.schema_meaning.hazard, id.hazard_schema());
        assert!(contract
            .preconditions
            .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1)));
        assert!(contract
            .preconditions
            .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(id)));
        for precondition in &contract.preconditions {
            match precondition {
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::EntityPresent(key) => {
                    assert!(contract.read.entities.contains(key));
                }
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::RelationPresent(key) => {
                    assert!(contract.read.relations.contains(key));
                }
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Schema(_)
                | worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::Branch(_)
                | worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaPrecondition::DeltaNotAccepted(_) => {}
            }
        }
    }
}

#[test]
fn every_delta_has_an_independent_exact_effect_proof() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    for id in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ALL {
        let child = baseline
            .branch
            .fork(id.branch(), worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
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
                .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationTargetChanges(key)));
            assert!(contract
                .postconditions
                .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::RelationSourceUnchanged(key)));
        }
        for path in &expected.fields {
            assert!(contract.footprint.fields.contains(path));
            let covered = contract
                .postconditions
                .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::FieldChanges(*path))
                || (path.field == Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::FieldKey::Posture)
                    && contract
                        .postconditions
                        .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::EntityPostureChanges(path.entity)));
            assert!(covered, "{id:?} omitted postcondition {path:?}");
        }
        if let Some(schema) = expected.schema {
            assert!(contract
                .postconditions
                .contains(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta::DeltaPostcondition::SchemaChanges(schema)));
        }
        assert_all_unlisted_fields_unchanged(&before, &after, &expected.fields);
    }
}

#[allow(unused_imports)]
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::RelationEdge, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::relation_comparison::validate_relation_vector};

#[test]
fn comparator_reports_missing_write_and_sibling_leak_separately() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let next = apply(&branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap();
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&next);

    let mut missing = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    missing.remove_entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity());
    assert!(matches!(
        compare(&expected, &missing).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingEntity(key) if key == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()
    ));

    let mut missing_relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    let relation_key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    missing_relation.remove_relation(relation_key);
    assert!(matches!(
        compare(&expected, &missing_relation).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingRelation(key) if key == relation_key
    ));

    let mut sibling = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    sibling.set_branch(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance);
    assert!(matches!(
        compare(&expected, &sibling).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::SiblingFactLeak { .. }
    ));
}

#[test]
fn comparator_reports_floating_head_and_wrong_ancestry() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&branch);
    let mut floating = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    floating.set_branch(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating);
    assert!(matches!(
        compare(&expected, &floating).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::FloatingBranchSelection(_)
    ));

    let mut wrong_parent = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    wrong_parent.set_parent(Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs));
    assert!(matches!(
        compare(&expected, &wrong_parent).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::WrongAncestry { .. }
    ));

    let mut wrong_lineage = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    wrong_lineage.set_lineage(vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs]);
    assert!(matches!(
        compare(&expected, &wrong_lineage).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::WrongAncestry { .. }
    ));
}

#[test]
fn relation_vectors_report_duplicate_and_illegal_endpoint() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let edge = *expected.relations.values().next().unwrap();
    let duplicate = validate_relation_vector(
        &baseline.definition.schema,
        &[edge, edge],
        &expected.entities,
    )
    .unwrap_err();
    assert!(matches!(
        duplicate.mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::DuplicateRelation(key) if key == edge.key
    ));

    let illegal = RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort, 100_000),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity(),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint.entity(),
    };
    let error =
        validate_relation_vector(&baseline.definition.schema, &[illegal], &expected.entities)
            .unwrap_err();
    assert!(matches!(
        error.mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::IllegalEndpoint(_)
    ));
}

#[test]
fn relation_vector_map_parity_rejects_omission_replacement_and_extra_edge() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);

    let mut missing = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    missing.remove_relation_from_vector(key);
    assert!(matches!(
        compare(&expected, &missing).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::MissingRelationVector(observed) if observed == key
    ));

    let mut replacement = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    replacement.repoint_relation_vector(
        key,
        Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity()),
        Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian.entity()),
    );
    assert!(matches!(
        compare(&expected, &replacement).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationVectorValue(observed) if observed == key
    ));

    let mut extra = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    extra.append_relation_to_vector(RelationEdge {
        key: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 99_999),
        source: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel, 1),
        target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Beacon.entity(),
    });
    assert!(matches!(
        compare(&expected, &extra).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::UnexpectedRelationVector(observed)
            if observed == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth, 99_999)
    ));
}

#[test]
fn comparator_reports_schema_meaning_drift_and_accepts_vector_permutation() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut drift = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    drift.schema = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2;
    assert!(matches!(
        compare(&expected, &drift).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::SchemaMeaning {
            expected: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            observed: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2,
        }
    ));

    let mut permuted = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    permuted.relation_vector.reverse();
    assert_eq!(compare(&expected, &permuted), Ok(()));
}

#[test]
fn a_valid_observation_compares_successfully_before_mutation_twins() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let observed = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    assert_eq!(compare(&expected, &observed), Ok(()));
}

#[test]
fn comparator_covers_absence_source_and_accepted_history_axes() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let entity = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MachineParts.entity();
    let relation = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let absent_branch = OracleBranch {
        state: baseline
            .branch
            .state
            .remove_entity(entity)
            .remove_relation(relation),
        ancestry: baseline.branch.ancestry.clone(),
    };
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&absent_branch);
    assert_eq!(
        compare(
            &expected,
            &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected)
        ),
        Ok(())
    );

    let mut wrong_entity_absence = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    wrong_entity_absence.absent_entities.clear();
    assert!(matches!(
        compare(&expected, &wrong_entity_absence).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::EntityAbsence(key) if key == entity
    ));

    let mut wrong_relation_absence = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    wrong_relation_absence.absent_relations.clear();
    assert!(matches!(
        compare(&expected, &wrong_relation_absence).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationAbsence(key) if key == relation
    ));

    let operating = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&operating.branch);
    let mut source = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    source.rebase_relation(relation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian.entity());
    assert!(matches!(
        compare(&expected, &source).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::RelationSource(key) if key == relation
    ));

    let mut accepted = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    accepted.set_accepted(vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo]);
    assert!(matches!(
        compare(&expected, &accepted).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::AcceptedDeltaOrder { .. }
    ));

    let mut history = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    history.set_history(vec![AcceptedDelta {
        branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
    }]);
    assert!(matches!(
        compare(&expected, &history).unwrap_err().mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::AcceptedHistory { .. }
    ));
}

use crate::relational_certification::world::supply_chain::*;

#[test]
fn comparator_reports_missing_write_and_sibling_leak_separately() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let next = apply(&branch, DeltaId::StormRerouteAurora).unwrap();
    let expected = ExpectedSupplyChainObservation::from_branch(&next);

    let mut missing = ObservedSupplyChainState::from_expected(&expected);
    missing.remove_entity(Anchor::AuroraEastbound.entity());
    assert!(matches!(
        compare(&expected, &missing).unwrap_err().mismatch,
        ComparisonMismatch::MissingEntity(key) if key == Anchor::AuroraEastbound.entity()
    ));

    let mut missing_relation = ObservedSupplyChainState::from_expected(&expected);
    let relation_key = RelationKey::new(RelationKind::CallAtPort, 1);
    missing_relation.remove_relation(relation_key);
    assert!(matches!(
        compare(&expected, &missing_relation).unwrap_err().mismatch,
        ComparisonMismatch::MissingRelation(key) if key == relation_key
    ));

    let mut sibling = ObservedSupplyChainState::from_expected(&expected);
    sibling.set_branch(BranchLabel::Maintenance);
    assert!(matches!(
        compare(&expected, &sibling).unwrap_err().mismatch,
        ComparisonMismatch::SiblingFactLeak { .. }
    ));
}

#[test]
fn comparator_reports_floating_head_and_wrong_ancestry() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let expected = ExpectedSupplyChainObservation::from_branch(&branch);
    let mut floating = ObservedSupplyChainState::from_expected(&expected);
    floating.set_branch(BranchLabel::Operating);
    assert!(matches!(
        compare(&expected, &floating).unwrap_err().mismatch,
        ComparisonMismatch::FloatingBranchSelection(_)
    ));

    let mut wrong_parent = ObservedSupplyChainState::from_expected(&expected);
    wrong_parent.set_parent(Some(BranchLabel::Customs));
    assert!(matches!(
        compare(&expected, &wrong_parent).unwrap_err().mismatch,
        ComparisonMismatch::WrongAncestry { .. }
    ));

    let mut wrong_lineage = ObservedSupplyChainState::from_expected(&expected);
    wrong_lineage.set_lineage(vec![BranchLabel::Operating, BranchLabel::Customs]);
    assert!(matches!(
        compare(&expected, &wrong_lineage).unwrap_err().mismatch,
        ComparisonMismatch::WrongAncestry { .. }
    ));
}

#[test]
fn relation_vectors_report_duplicate_and_illegal_endpoint() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let edge = *expected.relations.values().next().unwrap();
    let duplicate = validate_relation_vector(
        &baseline.definition.schema,
        &[edge, edge],
        &expected.entities,
    )
    .unwrap_err();
    assert!(matches!(
        duplicate.mismatch,
        ComparisonMismatch::DuplicateRelation(key) if key == edge.key
    ));

    let illegal = RelationEdge {
        key: RelationKey::new(RelationKind::TerminalAtPort, 100_000),
        source: Anchor::Meridian.entity(),
        target: Anchor::Southpoint.entity(),
    };
    let error =
        validate_relation_vector(&baseline.definition.schema, &[illegal], &expected.entities)
            .unwrap_err();
    assert!(matches!(
        error.mismatch,
        ComparisonMismatch::IllegalEndpoint(_)
    ));
}

#[test]
fn relation_vector_map_parity_rejects_omission_replacement_and_extra_edge() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let key = RelationKey::new(RelationKind::CallAtPort, 1);

    let mut missing = ObservedSupplyChainState::from_expected(&expected);
    missing.remove_relation_from_vector(key);
    assert!(matches!(
        compare(&expected, &missing).unwrap_err().mismatch,
        ComparisonMismatch::MissingRelationVector(observed) if observed == key
    ));

    let mut replacement = ObservedSupplyChainState::from_expected(&expected);
    replacement.repoint_relation_vector(
        key,
        Some(Anchor::AuroraMeridian.entity()),
        Some(Anchor::Meridian.entity()),
    );
    assert!(matches!(
        compare(&expected, &replacement).unwrap_err().mismatch,
        ComparisonMismatch::RelationVectorValue(observed) if observed == key
    ));

    let mut extra = ObservedSupplyChainState::from_expected(&expected);
    extra.append_relation_to_vector(RelationEdge {
        key: RelationKey::new(RelationKind::VesselAssignedToBerth, 99_999),
        source: EntityKey::new(EntityKind::Vessel, 1),
        target: Anchor::Beacon.entity(),
    });
    assert!(matches!(
        compare(&expected, &extra).unwrap_err().mismatch,
        ComparisonMismatch::UnexpectedRelationVector(observed)
            if observed == RelationKey::new(RelationKind::VesselAssignedToBerth, 99_999)
    ));
}

#[test]
fn comparator_reports_schema_meaning_drift_and_accepts_vector_permutation() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let mut drift = ObservedSupplyChainState::from_expected(&expected);
    drift.schema = SchemaVersion::V2;
    assert!(matches!(
        compare(&expected, &drift).unwrap_err().mismatch,
        ComparisonMismatch::SchemaMeaning {
            expected: SchemaVersion::V1,
            observed: SchemaVersion::V2,
        }
    ));

    let mut permuted = ObservedSupplyChainState::from_expected(&expected);
    permuted.relation_vector.reverse();
    assert_eq!(compare(&expected, &permuted), Ok(()));
}

#[test]
fn a_valid_observation_compares_successfully_before_mutation_twins() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&baseline.branch);
    let observed = ObservedSupplyChainState::from_expected(&expected);
    assert_eq!(compare(&expected, &observed), Ok(()));
}

#[test]
fn comparator_covers_absence_source_and_accepted_history_axes() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let entity = Anchor::MachineParts.entity();
    let relation = RelationKey::new(RelationKind::CallAtPort, 1);
    let absent_branch = OracleBranch {
        state: baseline
            .branch
            .state
            .remove_entity(entity)
            .remove_relation(relation),
        ancestry: baseline.branch.ancestry.clone(),
    };
    let expected = ExpectedSupplyChainObservation::from_branch(&absent_branch);
    assert_eq!(
        compare(
            &expected,
            &ObservedSupplyChainState::from_expected(&expected)
        ),
        Ok(())
    );

    let mut wrong_entity_absence = ObservedSupplyChainState::from_expected(&expected);
    wrong_entity_absence.absent_entities.clear();
    assert!(matches!(
        compare(&expected, &wrong_entity_absence).unwrap_err().mismatch,
        ComparisonMismatch::EntityAbsence(key) if key == entity
    ));

    let mut wrong_relation_absence = ObservedSupplyChainState::from_expected(&expected);
    wrong_relation_absence.absent_relations.clear();
    assert!(matches!(
        compare(&expected, &wrong_relation_absence).unwrap_err().mismatch,
        ComparisonMismatch::RelationAbsence(key) if key == relation
    ));

    let operating = SupplyChainBaseline::operating(SupplyChainScale::court());
    let expected = ExpectedSupplyChainObservation::from_branch(&operating.branch);
    let mut source = ObservedSupplyChainState::from_expected(&expected);
    source.rebase_relation(relation, Anchor::AuroraMeridian.entity());
    assert!(matches!(
        compare(&expected, &source).unwrap_err().mismatch,
        ComparisonMismatch::RelationSource(key) if key == relation
    ));

    let mut accepted = ObservedSupplyChainState::from_expected(&expected);
    accepted.set_accepted(vec![DeltaId::HoldMedicalCargo]);
    assert!(matches!(
        compare(&expected, &accepted).unwrap_err().mismatch,
        ComparisonMismatch::AcceptedDeltaOrder { .. }
    ));

    let mut history = ObservedSupplyChainState::from_expected(&expected);
    history.set_history(vec![AcceptedDelta {
        branch: BranchLabel::Operating,
        delta: DeltaId::HoldMedicalCargo,
    }]);
    assert!(matches!(
        compare(&expected, &history).unwrap_err().mismatch,
        ComparisonMismatch::AcceptedHistory { .. }
    ));
}

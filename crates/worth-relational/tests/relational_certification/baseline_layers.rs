use crate::relational_certification::world::supply_chain::*;

#[test]
fn baseline_portfolio_is_derived_and_semantically_distinct() {
    let scale = SupplyChainScale::court();
    let empty = SupplyChainBaseline::empty(scale);
    let operating = SupplyChainBaseline::operating(scale);
    let contested = SupplyChainBaseline::contested(scale);
    let retention = SupplyChainBaseline::retention_pressure(scale);
    let version = SupplyChainBaseline::version_boundary(scale);

    assert_eq!(empty.name, BaselineName::EmptyInstallation);
    assert!(empty.definition.entities.is_empty());
    assert_eq!(operating.name, BaselineName::Operating);
    assert!(!operating.definition.entities.is_empty());
    assert_eq!(contested.name, BaselineName::ContestedPlanning);
    assert_eq!(contested.branch_intents.len(), 4);
    assert!(contested.branch.ancestry.accepted.is_empty());
    assert_eq!(contested.validate_branch_intents(), Ok(()));
    for intent in &contested.branch_intents {
        let child = contested.branch.fork(intent.branch, intent.parent).unwrap();
        assert_eq!(child.state, contested.branch.state);
        assert_eq!(child.ancestry.parent, Some(BranchLabel::Operating));
    }
    assert_eq!(retention.name, BaselineName::RetentionPressure);
    assert_eq!(retention.retention_obligations.len(), 5);
    assert_eq!(version.name, BaselineName::VersionBoundary);
    assert_eq!(version.pre_upgrade_schema, Some(SchemaVersion::V1));
    assert_eq!(version.post_upgrade_schema, Some(SchemaVersion::V2));
    assert!(version.branch_intents.is_empty());
    assert_eq!(version.definition, operating.definition);
}

#[test]
fn malformed_branch_intents_fail_with_typed_legality_errors() {
    let mut baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    baseline.branch_intents = vec![BranchCreationIntent {
        branch: BranchLabel::Operating,
        parent: BranchLabel::Operating,
    }];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(BranchIntentError::ChildEqualsParent(BranchLabel::Operating))
    );

    baseline.branch_intents = vec![BranchCreationIntent {
        branch: BranchLabel::Operating,
        parent: BranchLabel::Customs,
    }];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(BranchIntentError::CurrentBranch(BranchLabel::Operating))
    );

    baseline.branch_intents = vec![BranchCreationIntent {
        branch: BranchLabel::Storm,
        parent: BranchLabel::Customs,
    }];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(BranchIntentError::ParentUnavailable {
            branch: BranchLabel::Storm,
            parent: BranchLabel::Customs,
        })
    );

    baseline.branch_intents = vec![
        BranchCreationIntent {
            branch: BranchLabel::Storm,
            parent: BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: BranchLabel::Storm,
            parent: BranchLabel::Operating,
        },
    ];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(BranchIntentError::DuplicatePair(
            BranchLabel::Storm,
            BranchLabel::Operating,
        ))
    );

    baseline.branch_intents[1].parent = BranchLabel::Customs;
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(BranchIntentError::DuplicateChild(BranchLabel::Storm))
    );
}

#[test]
fn retention_obligations_are_descriptive_not_runtime_leases() {
    let baseline = SupplyChainBaseline::retention_pressure(SupplyChainScale::court());
    let kinds: Vec<_> = baseline
        .retention_obligations
        .iter()
        .map(|obligation| obligation.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            RetentionObligationKind::Snapshot,
            RetentionObligationKind::Observation,
            RetentionObligationKind::Transaction,
            RetentionObligationKind::Candidate,
            RetentionObligationKind::ExternalBasis,
        ]
    );
    assert!(baseline
        .retention_obligations
        .iter()
        .all(|obligation| obligation.target == BranchLabel::Maintenance
            && obligation.ancestor_path.first() == Some(&BranchLabel::Operating)));
    assert_eq!(baseline.validate_retention_obligations(), Ok(()));
    let mut malformed = baseline.clone();
    malformed.retention_obligations[0].ancestor_path.clear();
    assert!(matches!(
        malformed.validate_retention_obligations(),
        Err(RetentionObligationError::EmptyPath(_))
    ));

    malformed.retention_obligations[0].ancestor_path =
        vec![BranchLabel::Operating, BranchLabel::HazardV2];
    assert!(matches!(
        malformed.validate_retention_obligations(),
        Err(RetentionObligationError::UnknownAncestorPath {
            kind: RetentionObligationKind::Snapshot,
            parent: BranchLabel::Operating,
            child: BranchLabel::HazardV2,
        })
    ));

    malformed.retention_obligations[0].ancestor_path = vec![BranchLabel::Operating];
    malformed.retention_obligations[0].target = BranchLabel::HazardV2;
    assert!(matches!(
        malformed.validate_retention_obligations(),
        Err(RetentionObligationError::UnknownTarget {
            kind: RetentionObligationKind::Snapshot,
            target: BranchLabel::HazardV2,
        })
    ));
}

#[test]
fn retention_pressure_can_record_entity_and_relation_deletion_markers() {
    let baseline = SupplyChainBaseline::retention_pressure(SupplyChainScale::court());
    let entity = Anchor::MachineParts.entity();
    let relation = RelationKey::new(RelationKind::CargoBookedOnVoyage, 0);
    let state = baseline
        .branch
        .state
        .remove_entity(entity)
        .remove_relation(relation);
    assert_eq!(
        state.absence_marker(entity),
        Some((AbsenceKind::Entity, entity))
    );
    assert_eq!(
        state.relation_absence_marker(relation),
        Some((AbsenceKind::Relation, relation))
    );
    let branch = OracleBranch {
        state,
        ancestry: baseline.branch.ancestry,
    };
    let expected = ExpectedSupplyChainObservation::from_branch(&branch);
    assert_eq!(
        compare(
            &expected,
            &ObservedSupplyChainState::from_expected(&expected)
        ),
        Ok(())
    );
}

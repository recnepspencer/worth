pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchCreationIntent, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale};

#[test]
fn baseline_portfolio_is_derived_and_semantically_distinct() {
    let scale = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court();
    let empty = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::empty(scale);
    let operating = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(scale);
    let contested = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::contested(scale);
    let retention = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::retention_pressure(scale);
    let version = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::version_boundary(scale);

    assert_eq!(empty.name, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::EmptyInstallation);
    assert!(empty.definition.entities.is_empty());
    assert_eq!(operating.name, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::Operating);
    assert!(!operating.definition.entities.is_empty());
    assert_eq!(contested.name, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::ContestedPlanning);
    assert_eq!(contested.branch_intents.len(), 4);
    assert!(contested.branch.ancestry.accepted.is_empty());
    assert_eq!(contested.validate_branch_intents(), Ok(()));
    for intent in &contested.branch_intents {
        let child = contested.branch.fork(intent.branch, intent.parent).unwrap();
        assert_eq!(child.state, contested.branch.state);
        assert_eq!(child.ancestry.parent, Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating));
    }
    assert_eq!(retention.name, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::RetentionPressure);
    assert_eq!(retention.retention_obligations.len(), 5);
    assert_eq!(version.name, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName::VersionBoundary);
    assert_eq!(version.pre_upgrade_schema, Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1));
    assert_eq!(version.post_upgrade_schema, Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2));
    assert!(version.branch_intents.is_empty());
    assert_eq!(version.definition, operating.definition);
}

#[test]
fn malformed_branch_intents_fail_with_typed_legality_errors() {
    let mut baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    baseline.branch_intents = vec![BranchCreationIntent {
        branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
    }];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::ChildEqualsParent(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating))
    );

    baseline.branch_intents = vec![BranchCreationIntent {
        branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
    }];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::CurrentBranch(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating))
    );

    baseline.branch_intents = vec![BranchCreationIntent {
        branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
        parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
    }];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::ParentUnavailable {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
        })
    );

    baseline.branch_intents = vec![
        BranchCreationIntent {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        },
        BranchCreationIntent {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        },
    ];
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::DuplicatePair(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
        ))
    );

    baseline.branch_intents[1].parent = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs;
    assert_eq!(
        baseline.validate_branch_intents(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::BranchIntentError::DuplicateChild(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm))
    );
}

#[test]
fn retention_obligations_are_descriptive_not_runtime_leases() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::retention_pressure(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let kinds: Vec<_> = baseline
        .retention_obligations
        .iter()
        .map(|obligation| obligation.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Snapshot,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Observation,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Transaction,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Candidate,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::ExternalBasis,
        ]
    );
    assert!(baseline
        .retention_obligations
        .iter()
        .all(|obligation| obligation.target == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance
            && obligation.ancestor_path.first() == Some(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)));
    assert_eq!(baseline.validate_retention_obligations(), Ok(()));
    let mut malformed = baseline.clone();
    malformed.retention_obligations[0].ancestor_path.clear();
    assert!(matches!(
        malformed.validate_retention_obligations(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::EmptyPath(_))
    ));

    malformed.retention_obligations[0].ancestor_path =
        vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2];
    assert!(matches!(
        malformed.validate_retention_obligations(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::UnknownAncestorPath {
            kind: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Snapshot,
            parent: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
            child: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2,
        })
    ));

    malformed.retention_obligations[0].ancestor_path = vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating];
    malformed.retention_obligations[0].target = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2;
    assert!(matches!(
        malformed.validate_retention_obligations(),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationError::UnknownTarget {
            kind: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::RetentionObligationKind::Snapshot,
            target: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2,
        })
    ));
}

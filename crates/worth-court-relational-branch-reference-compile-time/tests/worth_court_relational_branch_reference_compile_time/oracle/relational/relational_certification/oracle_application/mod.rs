pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleAncestry, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply_from_parent, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::reject_duplicate_relation, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema};

#[test]
fn every_named_delta_applies_to_the_correct_semantic_branch() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let cases = [
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::SouthpointExpansion,
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::CompetingArrival,
        ),
        (
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Inspection,
        ),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire),
        (worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2),
    ];
    for (delta, branch_label) in cases {
        let branch = baseline
            .branch
            .fork(branch_label, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
            .unwrap();
        let applied = apply(&branch, delta).unwrap();
        assert_eq!(applied.ancestry.branch, branch_label);
        assert_eq!(applied.ancestry.accepted, vec![delta]);
    }
}

#[test]
fn failed_delta_application_leaves_input_unchanged_and_typed() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let missing = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0);
    let broken = OracleBranch {
        state: branch.state.remove_entity(missing),
        ancestry: branch.ancestry.clone(),
    };
    let before = broken.clone();
    let error = apply(&broken, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap_err();
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::MissingEntity {
            delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
            key
        } if key == missing
    ));
    assert_eq!(broken, before);
}

#[test]
fn schema_transition_is_typed_precedence_and_non_mutating() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let v1 = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let v2 = apply(&v1, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2).unwrap();
    assert_eq!(
        v2.state.schema,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2)
    );
    let before = v2.clone();
    let error = apply(&v2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2).unwrap_err();
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::InvalidSchemaTransition {
            expected: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
            observed: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2,
        }
    ));
    assert_eq!(v2, before);

    for delta in worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ALL {
        if delta == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 {
            continue;
        }
        let mut candidate = v2.clone();
        candidate.ancestry.branch = delta.branch();
        let before = candidate.clone();
        let error = apply(&candidate, delta).unwrap_err();
        assert!(matches!(
            error,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::InvalidSchemaTransition {
                expected: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1,
                observed: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2,
            }
        ));
        assert_eq!(candidate, before);
    }
}

#[test]
fn successful_application_rejects_an_invalid_preexisting_topology() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::MedicalHold, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let relation_key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort, 0);
    let mut invalid = branch.clone();
    let mut edge = invalid.state.relation(relation_key).unwrap().to_owned();
    edge.target = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, u32::MAX);
    invalid.state.relations.insert(relation_key, edge);
    let before = invalid.clone();
    let error = apply(&invalid, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo).unwrap_err();
    assert!(matches!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::InvalidPostState(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaError::InvalidEndpoint { .. })
    ));
    assert_eq!(invalid, before);
}

#[test]
fn sibling_delta_and_wrong_parent_are_distinct_failures() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let maintenance = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let sibling = apply(&maintenance, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap_err();
    assert!(matches!(
        sibling,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::SiblingFactLeak { .. }
    ));

    let forged = OracleBranch {
        state: maintenance.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs),
            lineage: vec![
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    let ancestry =
        apply_from_parent(&baseline.branch, &forged, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap_err();
    assert!(matches!(ancestry, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry(_)));

    let storm_parent = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let forged_lineage = OracleBranch {
        state: storm_parent.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
            lineage: vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(&storm_parent, &forged_lineage, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::LineageMismatch { .. }
        ))
    ));

    let forged_suffix = OracleBranch {
        state: storm_parent.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
            lineage: vec![
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(&storm_parent, &forged_suffix, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::LineageMismatch { .. }
        ))
    ));

    let forged_duplicate = OracleBranch {
        state: storm_parent.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
            lineage: vec![
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(
            &storm_parent,
            &forged_duplicate,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::LineageMismatch { .. }
        ))
    ));
}

#[test]
fn malformed_parent_root_and_duplicate_labels_are_independently_rejected() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let malformed_root_parent = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
            lineage: vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    let child_of_malformed_root = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
            lineage: vec![
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(
            &malformed_root_parent,
            &child_of_malformed_root,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall,
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::LineageMismatch { .. }
        ))
    ));

    let malformed_duplicate_parent = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating),
            lineage: vec![
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    let child_of_malformed_duplicate = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            parent: Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm),
            lineage: vec![
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Rewire,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(
            &malformed_duplicate_parent,
            &child_of_malformed_duplicate,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall,
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::WrongAncestry(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::LineageMismatch { .. }
        ))
    ));
}

#[test]
fn duplicate_relation_is_denied_before_oracle_mutation() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1);
    let error = reject_duplicate_relation(&baseline.branch, key).unwrap_err();
    assert_eq!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::DuplicateRelation(key));
}

#[test]
fn duplicate_delta_reapplication_is_typed_and_non_mutating() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let applied = apply(&branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap();
    let error = apply(&applied, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap_err();
    assert_eq!(
        error,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::DuplicateDelta(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora)
    );
    assert_eq!(applied.ancestry.accepted, vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora]);
}

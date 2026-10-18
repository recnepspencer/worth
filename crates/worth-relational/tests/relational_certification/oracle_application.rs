use crate::relational_certification::world::supply_chain::*;

#[test]
fn every_named_delta_applies_to_the_correct_semantic_branch() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let cases = [
        (DeltaId::StormRerouteAurora, BranchLabel::Storm),
        (DeltaId::MaintainAtlasBerth, BranchLabel::Maintenance),
        (DeltaId::HoldMedicalCargo, BranchLabel::MedicalHold),
        (
            DeltaId::ExpandSouthpointCapacity,
            BranchLabel::SouthpointExpansion,
        ),
        (
            DeltaId::CompetingAuroraArrival,
            BranchLabel::CompetingArrival,
        ),
        (
            DeltaId::RetireAtlasWhileInspectingAurora,
            BranchLabel::Inspection,
        ),
        (DeltaId::RewireAuroraPortCall, BranchLabel::Rewire),
        (DeltaId::AdoptHazardClassificationV2, BranchLabel::HazardV2),
    ];
    for (delta, branch_label) in cases {
        let branch = baseline
            .branch
            .fork(branch_label, BranchLabel::Operating)
            .unwrap();
        let applied = apply(&branch, delta).unwrap();
        assert_eq!(applied.ancestry.branch, branch_label);
        assert_eq!(applied.ancestry.accepted, vec![delta]);
    }
}

#[test]
fn failed_delta_application_leaves_input_unchanged_and_typed() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let missing = EntityKey::new(EntityKind::Voyage, 0);
    let broken = OracleBranch {
        state: branch.state.remove_entity(missing),
        ancestry: branch.ancestry.clone(),
    };
    let before = broken.clone();
    let error = apply(&broken, DeltaId::StormRerouteAurora).unwrap_err();
    assert!(matches!(
        error,
        OracleApplicationError::MissingEntity {
            delta: DeltaId::StormRerouteAurora,
            key
        } if key == missing
    ));
    assert_eq!(broken, before);
}

#[test]
fn schema_transition_is_typed_precedence_and_non_mutating() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let v1 = baseline
        .branch
        .fork(BranchLabel::HazardV2, BranchLabel::Operating)
        .unwrap();
    let v2 = apply(&v1, DeltaId::AdoptHazardClassificationV2).unwrap();
    assert_eq!(
        v2.state.schema,
        SupplyChainSchema::canonical(SchemaVersion::V2)
    );
    let before = v2.clone();
    let error = apply(&v2, DeltaId::AdoptHazardClassificationV2).unwrap_err();
    assert!(matches!(
        error,
        OracleApplicationError::InvalidSchemaTransition {
            expected: SchemaVersion::V1,
            observed: SchemaVersion::V2,
        }
    ));
    assert_eq!(v2, before);

    for delta in DeltaId::ALL {
        if delta == DeltaId::AdoptHazardClassificationV2 {
            continue;
        }
        let mut candidate = v2.clone();
        candidate.ancestry.branch = delta.branch();
        let before = candidate.clone();
        let error = apply(&candidate, delta).unwrap_err();
        assert!(matches!(
            error,
            OracleApplicationError::InvalidSchemaTransition {
                expected: SchemaVersion::V1,
                observed: SchemaVersion::V2,
            }
        ));
        assert_eq!(candidate, before);
    }
}

#[test]
fn successful_application_rejects_an_invalid_preexisting_topology() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(BranchLabel::MedicalHold, BranchLabel::Operating)
        .unwrap();
    let relation_key = RelationKey::new(RelationKind::TerminalAtPort, 0);
    let mut invalid = branch.clone();
    let mut edge = invalid.state.relation(relation_key).unwrap().to_owned();
    edge.target = EntityKey::new(EntityKind::Port, u32::MAX);
    invalid.state.relations.insert(relation_key, edge);
    let before = invalid.clone();
    let error = apply(&invalid, DeltaId::HoldMedicalCargo).unwrap_err();
    assert!(matches!(
        error,
        OracleApplicationError::InvalidPostState(SchemaError::InvalidEndpoint { .. })
    ));
    assert_eq!(invalid, before);
}

#[test]
fn sibling_delta_and_wrong_parent_are_distinct_failures() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let maintenance = baseline
        .branch
        .fork(BranchLabel::Maintenance, BranchLabel::Operating)
        .unwrap();
    let sibling = apply(&maintenance, DeltaId::StormRerouteAurora).unwrap_err();
    assert!(matches!(
        sibling,
        OracleApplicationError::SiblingFactLeak { .. }
    ));

    let forged = OracleBranch {
        state: maintenance.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Storm,
            parent: Some(BranchLabel::Customs),
            lineage: vec![
                BranchLabel::Operating,
                BranchLabel::Customs,
                BranchLabel::Storm,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    let ancestry =
        apply_from_parent(&baseline.branch, &forged, DeltaId::StormRerouteAurora).unwrap_err();
    assert!(matches!(ancestry, OracleApplicationError::WrongAncestry(_)));

    let storm_parent = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let forged_lineage = OracleBranch {
        state: storm_parent.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Customs,
            parent: Some(BranchLabel::Storm),
            lineage: vec![BranchLabel::Operating, BranchLabel::Customs],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(&storm_parent, &forged_lineage, DeltaId::MaintainAtlasBerth),
        Err(OracleApplicationError::WrongAncestry(
            AncestryError::LineageMismatch { .. }
        ))
    ));

    let forged_suffix = OracleBranch {
        state: storm_parent.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Customs,
            parent: Some(BranchLabel::Storm),
            lineage: vec![
                BranchLabel::Operating,
                BranchLabel::Storm,
                BranchLabel::Customs,
                BranchLabel::Rewire,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(&storm_parent, &forged_suffix, DeltaId::MaintainAtlasBerth),
        Err(OracleApplicationError::WrongAncestry(
            AncestryError::LineageMismatch { .. }
        ))
    ));

    let forged_duplicate = OracleBranch {
        state: storm_parent.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Customs,
            parent: Some(BranchLabel::Storm),
            lineage: vec![
                BranchLabel::Operating,
                BranchLabel::Storm,
                BranchLabel::Storm,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(
            &storm_parent,
            &forged_duplicate,
            DeltaId::MaintainAtlasBerth
        ),
        Err(OracleApplicationError::WrongAncestry(
            AncestryError::LineageMismatch { .. }
        ))
    ));
}

#[test]
fn malformed_parent_root_and_duplicate_labels_are_independently_rejected() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let malformed_root_parent = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Storm,
            parent: Some(BranchLabel::Operating),
            lineage: vec![BranchLabel::Customs, BranchLabel::Storm],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    let child_of_malformed_root = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Rewire,
            parent: Some(BranchLabel::Storm),
            lineage: vec![
                BranchLabel::Customs,
                BranchLabel::Storm,
                BranchLabel::Rewire,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(
            &malformed_root_parent,
            &child_of_malformed_root,
            DeltaId::RewireAuroraPortCall,
        ),
        Err(OracleApplicationError::WrongAncestry(
            AncestryError::LineageMismatch { .. }
        ))
    ));

    let malformed_duplicate_parent = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Storm,
            parent: Some(BranchLabel::Operating),
            lineage: vec![
                BranchLabel::Operating,
                BranchLabel::Storm,
                BranchLabel::Storm,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    let child_of_malformed_duplicate = OracleBranch {
        state: baseline.branch.state.clone(),
        ancestry: OracleAncestry {
            branch: BranchLabel::Rewire,
            parent: Some(BranchLabel::Storm),
            lineage: vec![
                BranchLabel::Operating,
                BranchLabel::Storm,
                BranchLabel::Storm,
                BranchLabel::Rewire,
            ],
            accepted: Vec::new(),
            history: Vec::new(),
        },
    };
    assert!(matches!(
        apply_from_parent(
            &malformed_duplicate_parent,
            &child_of_malformed_duplicate,
            DeltaId::RewireAuroraPortCall,
        ),
        Err(OracleApplicationError::WrongAncestry(
            AncestryError::LineageMismatch { .. }
        ))
    ));
}

#[test]
fn duplicate_relation_is_denied_before_oracle_mutation() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let key = RelationKey::new(RelationKind::CallAtPort, 1);
    let error = reject_duplicate_relation(&baseline.branch, key).unwrap_err();
    assert_eq!(error, OracleApplicationError::DuplicateRelation(key));
}

#[test]
fn duplicate_delta_reapplication_is_typed_and_non_mutating() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let applied = apply(&branch, DeltaId::StormRerouteAurora).unwrap();
    let error = apply(&applied, DeltaId::StormRerouteAurora).unwrap_err();
    assert_eq!(
        error,
        OracleApplicationError::DuplicateDelta(DeltaId::StormRerouteAurora)
    );
    assert_eq!(applied.ancestry.accepted, vec![DeltaId::StormRerouteAurora]);
}

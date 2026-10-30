pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AcceptedDelta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::apply, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_digest::digest};

#[test]
fn sibling_branches_share_immutable_ancestor_but_not_lineage() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let maintenance = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    assert_eq!(storm.state, maintenance.state);
    assert_ne!(storm.ancestry.branch, maintenance.ancestry.branch);
    assert_eq!(storm.ancestry.parent, maintenance.ancestry.parent);
    assert_eq!(
        storm.ancestry.lineage,
        vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm]
    );
    assert!(storm.ancestry.history.is_empty());
    assert_eq!(
        storm.ancestry.common_ancestor(&maintenance.ancestry),
        Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
    );
}

#[test]
fn nested_forks_retain_recursive_lineage_and_reject_reuse() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let customs = storm
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm)
        .unwrap();
    assert_eq!(
        customs.ancestry.lineage,
        vec![
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs
        ]
    );
    let storm = apply(&storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap();
    let customs = storm
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm)
        .unwrap();
    assert_eq!(
        customs.ancestry.history,
        vec![AcceptedDelta {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
        }]
    );
    assert!(matches!(
        customs.fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::BranchAlreadyExists(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating))
    ));
}

#[test]
fn wrong_parent_is_checked_even_when_domain_state_is_equal() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let child = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let wrong_parent = child.ancestry.parent.map(|_| worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs).unwrap();
    let error = child.expects_parent(wrong_parent).unwrap_err();
    assert!(matches!(error, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::ParentMismatch { .. }));
}

#[test]
fn accepted_delta_order_is_semantic_ancestry() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let first = apply(&branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora).unwrap();
    let second = apply(&first, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival).unwrap_err();
    assert!(matches!(
        second,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::application::OracleApplicationError::SiblingFactLeak { .. }
    ));
    assert_eq!(first.ancestry.accepted, vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora]);
}

#[test]
fn ordered_history_fixture_preserves_order_without_fabricating_domain_replay() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let first = baseline
        .branch
        .ancestry
        .record_history(&[
            AcceptedDelta {
                branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
            },
            AcceptedDelta {
                branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating,
                delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
            },
        ])
        .unwrap();
    let second = storm
        .ancestry
        .record_history(&[
            AcceptedDelta {
                branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
            },
            AcceptedDelta {
                branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
                delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
            },
        ])
        .unwrap();
    assert_ne!(first.accepted, second.accepted);
    assert_eq!(first.branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating);
    assert_eq!(second.branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm);
    assert_eq!(first.common_ancestor(&second), Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating));
}

#[test]
fn history_owner_and_digest_are_branch_sensitive() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
        .unwrap();
    let recorded = storm
        .ancestry
        .record_history(&[AcceptedDelta {
            branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
            delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
        }])
        .unwrap();
    let forged_owner = storm.ancestry.record_history(&[AcceptedDelta {
        branch: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs,
        delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    }]);
    assert_eq!(
        forged_owner,
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::AncestryError::HistoryOwnerUnavailable(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs))
    );

    let branch = OracleBranch {
        state: storm.state.clone(),
        ancestry: recorded,
    };
    let expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&branch);
    let mut observed = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&expected);
    let mut changed_history = expected.ancestry.history.clone();
    changed_history[0].branch = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs;
    observed.set_history(changed_history);
    let failure = compare(&expected, &observed).unwrap_err();
    assert!(matches!(
        failure.mismatch,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ComparisonMismatch::AcceptedHistory { .. }
    ));

    let mut forged = branch.clone();
    forged.ancestry.history[0].branch = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Customs;
    assert_ne!(
        digest(&expected),
        digest(&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&forged))
    );
}

#[test]
fn sibling_mutation_court_proves_branch_local_fate_and_unchanged_ancestor() {
    let baseline = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::SupplyChainBaseline::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let ancestor = baseline.branch.clone();
    let storm = apply(
        &baseline
            .branch
            .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
            .unwrap(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    )
    .unwrap();
    let maintenance = apply(
        &baseline
            .branch
            .fork(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
            .unwrap(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    )
    .unwrap();
    assert_eq!(baseline.branch, ancestor);
    assert_eq!(storm.ancestry.parent, Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating));
    assert_eq!(maintenance.ancestry.parent, Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating));
    assert_eq!(storm.ancestry.accepted, vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora]);
    assert_eq!(
        maintenance.ancestry.accepted,
        vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth]
    );
    assert_ne!(
        storm.state.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity()),
        maintenance.state.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound.entity())
    );
    assert_ne!(
        storm.state.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity()),
        maintenance.state.entity(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas.entity())
    );
    assert_ne!(
        storm
            .state
            .relation(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1)),
        maintenance
            .state
            .relation(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort, 1))
    );
    let storm_expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&storm);
    let maintenance_expected = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&maintenance);
    assert_eq!(
        compare(
            &storm_expected,
            &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&storm_expected)
        ),
        Ok(())
    );
    assert_eq!(
        compare(
            &maintenance_expected,
            &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState::from_expected(&maintenance_expected)
        ),
        Ok(())
    );
}

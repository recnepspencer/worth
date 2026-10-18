use crate::relational_certification::world::supply_chain::*;

#[test]
fn sibling_branches_share_immutable_ancestor_but_not_lineage() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let maintenance = baseline
        .branch
        .fork(BranchLabel::Maintenance, BranchLabel::Operating)
        .unwrap();
    assert_eq!(storm.state, maintenance.state);
    assert_ne!(storm.ancestry.branch, maintenance.ancestry.branch);
    assert_eq!(storm.ancestry.parent, maintenance.ancestry.parent);
    assert_eq!(
        storm.ancestry.lineage,
        vec![BranchLabel::Operating, BranchLabel::Storm]
    );
    assert!(storm.ancestry.history.is_empty());
    assert_eq!(
        storm.ancestry.common_ancestor(&maintenance.ancestry),
        Some(BranchLabel::Operating)
    );
}

#[test]
fn nested_forks_retain_recursive_lineage_and_reject_reuse() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let customs = storm
        .fork(BranchLabel::Customs, BranchLabel::Storm)
        .unwrap();
    assert_eq!(
        customs.ancestry.lineage,
        vec![
            BranchLabel::Operating,
            BranchLabel::Storm,
            BranchLabel::Customs
        ]
    );
    let storm = apply(&storm, DeltaId::StormRerouteAurora).unwrap();
    let customs = storm
        .fork(BranchLabel::Customs, BranchLabel::Storm)
        .unwrap();
    assert_eq!(
        customs.ancestry.history,
        vec![AcceptedDelta {
            branch: BranchLabel::Storm,
            delta: DeltaId::StormRerouteAurora,
        }]
    );
    assert!(matches!(
        customs.fork(BranchLabel::Operating, BranchLabel::Customs),
        Err(AncestryError::BranchAlreadyExists(BranchLabel::Operating))
    ));
}

#[test]
fn wrong_parent_is_checked_even_when_domain_state_is_equal() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let child = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let wrong_parent = child.ancestry.parent.map(|_| BranchLabel::Customs).unwrap();
    let error = child.expects_parent(wrong_parent).unwrap_err();
    assert!(matches!(error, AncestryError::ParentMismatch { .. }));
}

#[test]
fn accepted_delta_order_is_semantic_ancestry() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let branch = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let first = apply(&branch, DeltaId::StormRerouteAurora).unwrap();
    let second = apply(&first, DeltaId::CompetingAuroraArrival).unwrap_err();
    assert!(matches!(
        second,
        OracleApplicationError::SiblingFactLeak { .. }
    ));
    assert_eq!(first.ancestry.accepted, vec![DeltaId::StormRerouteAurora]);
}

#[test]
fn ordered_history_fixture_preserves_order_without_fabricating_domain_replay() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let first = baseline
        .branch
        .ancestry
        .record_history(&[
            AcceptedDelta {
                branch: BranchLabel::Operating,
                delta: DeltaId::StormRerouteAurora,
            },
            AcceptedDelta {
                branch: BranchLabel::Operating,
                delta: DeltaId::MaintainAtlasBerth,
            },
        ])
        .unwrap();
    let second = storm
        .ancestry
        .record_history(&[
            AcceptedDelta {
                branch: BranchLabel::Storm,
                delta: DeltaId::MaintainAtlasBerth,
            },
            AcceptedDelta {
                branch: BranchLabel::Storm,
                delta: DeltaId::StormRerouteAurora,
            },
        ])
        .unwrap();
    assert_ne!(first.accepted, second.accepted);
    assert_eq!(first.branch, BranchLabel::Operating);
    assert_eq!(second.branch, BranchLabel::Storm);
    assert_eq!(first.common_ancestor(&second), Some(BranchLabel::Operating));
}

#[test]
fn history_owner_and_digest_are_branch_sensitive() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let storm = baseline
        .branch
        .fork(BranchLabel::Storm, BranchLabel::Operating)
        .unwrap();
    let recorded = storm
        .ancestry
        .record_history(&[AcceptedDelta {
            branch: BranchLabel::Storm,
            delta: DeltaId::StormRerouteAurora,
        }])
        .unwrap();
    let forged_owner = storm.ancestry.record_history(&[AcceptedDelta {
        branch: BranchLabel::Customs,
        delta: DeltaId::StormRerouteAurora,
    }]);
    assert_eq!(
        forged_owner,
        Err(AncestryError::HistoryOwnerUnavailable(BranchLabel::Customs))
    );

    let branch = OracleBranch {
        state: storm.state.clone(),
        ancestry: recorded,
    };
    let expected = ExpectedSupplyChainObservation::from_branch(&branch);
    let mut observed = ObservedSupplyChainState::from_expected(&expected);
    let mut changed_history = expected.ancestry.history.clone();
    changed_history[0].branch = BranchLabel::Customs;
    observed.set_history(changed_history);
    let failure = compare(&expected, &observed).unwrap_err();
    assert!(matches!(
        failure.mismatch,
        ComparisonMismatch::AcceptedHistory { .. }
    ));

    let mut forged = branch.clone();
    forged.ancestry.history[0].branch = BranchLabel::Customs;
    assert_ne!(
        digest(&expected),
        digest(&ExpectedSupplyChainObservation::from_branch(&forged))
    );
}

#[test]
fn sibling_mutation_court_proves_branch_local_fate_and_unchanged_ancestor() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let ancestor = baseline.branch.clone();
    let storm = apply(
        &baseline
            .branch
            .fork(BranchLabel::Storm, BranchLabel::Operating)
            .unwrap(),
        DeltaId::StormRerouteAurora,
    )
    .unwrap();
    let maintenance = apply(
        &baseline
            .branch
            .fork(BranchLabel::Maintenance, BranchLabel::Operating)
            .unwrap(),
        DeltaId::MaintainAtlasBerth,
    )
    .unwrap();
    assert_eq!(baseline.branch, ancestor);
    assert_eq!(storm.ancestry.parent, Some(BranchLabel::Operating));
    assert_eq!(maintenance.ancestry.parent, Some(BranchLabel::Operating));
    assert_eq!(storm.ancestry.accepted, vec![DeltaId::StormRerouteAurora]);
    assert_eq!(
        maintenance.ancestry.accepted,
        vec![DeltaId::MaintainAtlasBerth]
    );
    assert_ne!(
        storm.state.entity(Anchor::AuroraEastbound.entity()),
        maintenance.state.entity(Anchor::AuroraEastbound.entity())
    );
    assert_ne!(
        storm.state.entity(Anchor::Atlas.entity()),
        maintenance.state.entity(Anchor::Atlas.entity())
    );
    assert_ne!(
        storm
            .state
            .relation(RelationKey::new(RelationKind::CallAtPort, 1)),
        maintenance
            .state
            .relation(RelationKey::new(RelationKind::CallAtPort, 1))
    );
    let storm_expected = ExpectedSupplyChainObservation::from_branch(&storm);
    let maintenance_expected = ExpectedSupplyChainObservation::from_branch(&maintenance);
    assert_eq!(
        compare(
            &storm_expected,
            &ObservedSupplyChainState::from_expected(&storm_expected)
        ),
        Ok(())
    );
    assert_eq!(
        compare(
            &maintenance_expected,
            &ObservedSupplyChainState::from_expected(&maintenance_expected)
        ),
        Ok(())
    );
}

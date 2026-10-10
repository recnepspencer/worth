use super::support::*;

#[test]
fn pricing_shock_reference_matrix_preserves_semantic_truth_across_diagnostics_profiles() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let baseline = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-baseline"),
        resource_request,
    );
    let forensic = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::forensic(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-forensic"),
        resource_request,
    );

    assert_eq!(baseline.reference, forensic.reference);
    assert_eq!(baseline.replay, forensic.replay);
}

#[test]
fn pricing_shock_route_replay_preserves_canonical_main_branch_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let replay = capture_pricing_certification_matrix(
        BridgeRuntimePolicy::development(),
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-replay-control"),
        resource_request,
    )
    .replay;

    assert_eq!(
        replay.source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main")
    );
    assert_eq!(
        replay.source_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main")
    );
    assert!(!replay.route_identity.as_str().is_empty());
    assert!(!replay.invalidation_identity.as_str().is_empty());
}

#[test]
fn pricing_shock_duplicate_commit_identity_with_conflicting_route_meaning_is_detectable() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = build_pricing_runtime(
        pricing_reference_source_with_conflicting_commit_identity_for_route(),
        RecordingSignalBridgeSink::default(),
    );

    let route = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("conflicting duplicate commit identity should still route as retained truth");

    assert_eq!(
        route.result().result_summary().source_commit().as_str(),
        "commit:steel-main"
    );
    assert_eq!(
        route.result().receipt().snapshot_identity().as_str(),
        "snapshot:pricing-main"
    );
    assert_eq!(route.result().receipt().delivered_target_count(), 1);
    assert_eq!(
        route.result().artifact().invalidation_targets().targets()[0].signal_scope(),
        "price:scooter"
    );
}

#[test]
fn pricing_shock_duplicate_conflicting_commit_identity_permutation_sweep_is_detectable() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for (label, source, commit, expected_snapshot, expected_targets) in [
        (
            "steel-commit-rewritten-to-rubber",
            pricing_reference_source_with_conflicting_route_commit_items(
                "commit:steel-main",
                "patch:steel-main-conflicting-rubber",
                vec![BridgeCommittedPatchItem::with_target(
                    "component:rubber",
                    crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                        worth_foundational::facade::AspectLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            worth_foundational::facade::AspectKey::new("cost")
                                .expect("valid bridge patch aspect key"),
                        ),
                        worth_foundational::facade::CanonicalFieldPath::single(
                            worth_foundational::facade::FieldKey::new("usd".to_owned())
                                .expect("valid foundational field key"),
                        ),
                    ),
                )],
            ),
            "commit:steel-main",
            "snapshot:pricing-main",
            vec!["price:scooter"],
        ),
        (
            "rubber-commit-rewritten-to-steel",
            pricing_reference_source_with_conflicting_route_commit_items(
                "commit:rubber-main",
                "patch:rubber-main-conflicting-steel",
                vec![BridgeCommittedPatchItem::with_target(
                    "component:steel",
                    crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                        worth_foundational::facade::AspectLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            worth_foundational::facade::AspectKey::new("cost")
                                .expect("valid bridge patch aspect key"),
                        ),
                        worth_foundational::facade::CanonicalFieldPath::single(
                            worth_foundational::facade::FieldKey::new("usd".to_owned())
                                .expect("valid foundational field key"),
                        ),
                    ),
                )],
            ),
            "commit:rubber-main",
            "snapshot:pricing-main",
            vec!["price:bicycle", "price:wheelbarrow"],
        ),
        (
            "steel-commit-rewritten-to-combined-meaning",
            pricing_reference_source_with_conflicting_route_commit_items(
                "commit:steel-main",
                "patch:steel-main-conflicting-combined",
                vec![
                    BridgeCommittedPatchItem::with_target(
                        "component:steel",
                        crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                            worth_foundational::facade::AspectLocator::new(
                                worth_foundational::facade::LocatorAuthority::Authoritative,
                                worth_foundational::facade::AspectKey::new("cost")
                                    .expect("valid bridge patch aspect key"),
                            ),
                            worth_foundational::facade::CanonicalFieldPath::single(
                                worth_foundational::facade::FieldKey::new("usd".to_owned())
                                    .expect("valid foundational field key"),
                            ),
                        ),
                    ),
                    BridgeCommittedPatchItem::with_target(
                        "component:rubber",
                        crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                            worth_foundational::facade::AspectLocator::new(
                                worth_foundational::facade::LocatorAuthority::Authoritative,
                                worth_foundational::facade::AspectKey::new("cost")
                                    .expect("valid bridge patch aspect key"),
                            ),
                            worth_foundational::facade::CanonicalFieldPath::single(
                                worth_foundational::facade::FieldKey::new("usd".to_owned())
                                    .expect("valid foundational field key"),
                            ),
                        ),
                    ),
                ],
            ),
            "commit:steel-main",
            "snapshot:pricing-main",
            vec!["price:bicycle", "price:scooter", "price:wheelbarrow"],
        ),
    ] {
        let runtime = build_pricing_runtime(source, RecordingSignalBridgeSink::default());
        let route = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture(commit),
                execution,
            )
            .unwrap_or_else(|_| panic!("{label} should still route as retained truth"));

        assert_eq!(
            route.result().result_summary().source_commit().as_str(),
            commit
        );
        assert_eq!(
            route.result().receipt().snapshot_identity().as_str(),
            expected_snapshot
        );
        assert_eq!(
            route.result().receipt().delivered_target_count(),
            expected_targets.len(),
            "{label} should deliver the expected target width"
        );

        let mut actual_targets = route
            .result()
            .artifact()
            .invalidation_targets()
            .targets()
            .iter()
            .map(|target| target.signal_scope().to_owned())
            .collect::<Vec<_>>();
        actual_targets.sort();

        let mut expected_targets = expected_targets
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        expected_targets.sort();

        assert_eq!(
            actual_targets, expected_targets,
            "{label} should surface the conflicting retained meaning"
        );
    }
}

#[test]
fn pricing_shock_non_commuting_route_history_attack_fails_closed_on_replay() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let original_runtime = build_pricing_runtime(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
    );
    original_runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            execution,
        )
        .expect("original steel route should succeed before replay attack");
    let canonical_record = original_runtime
        .diagnostics()
        .last_canonical_route_record()
        .expect("original runtime should expose canonical route record");

    let restarted_runtime = build_pricing_runtime(
        pricing_reference_source_with_conflicting_commit_identity_for_route(),
        RecordingSignalBridgeSink::default(),
    );
    let error = restarted_runtime
        .replay_canonical_record(&canonical_record, execution)
        .expect_err("replay should reject non-commuting route history drift");

    assert!(!error.to_string().is_empty());
    let failure_record = restarted_runtime
        .diagnostics()
        .last_failure_record()
        .expect("replay failure should retain diagnostics");
    assert_eq!(failure_record.counters().route_replay_mismatch_count(), 1);
}

#[test]
fn pricing_shock_non_commuting_route_history_permutation_sweep_fails_closed() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for (label, clean_commit, mutated_source) in [
        (
            "steel-route-replayed-against-rubber-meaning",
            "commit:steel-main",
            pricing_reference_source_with_conflicting_route_commit_items(
                "commit:steel-main",
                "patch:steel-main-conflicting-rubber",
                vec![BridgeCommittedPatchItem::with_target(
                    "component:rubber",
                    crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                        worth_foundational::facade::AspectLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            worth_foundational::facade::AspectKey::new("cost")
                                .expect("valid bridge patch aspect key"),
                        ),
                        worth_foundational::facade::CanonicalFieldPath::single(
                            worth_foundational::facade::FieldKey::new("usd".to_owned())
                                .expect("valid foundational field key"),
                        ),
                    ),
                )],
            ),
        ),
        (
            "rubber-route-replayed-against-steel-meaning",
            "commit:rubber-main",
            pricing_reference_source_with_conflicting_route_commit_items(
                "commit:rubber-main",
                "patch:rubber-main-conflicting-steel",
                vec![BridgeCommittedPatchItem::with_target(
                    "component:steel",
                    crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                        worth_foundational::facade::AspectLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            worth_foundational::facade::AspectKey::new("cost")
                                .expect("valid bridge patch aspect key"),
                        ),
                        worth_foundational::facade::CanonicalFieldPath::single(
                            worth_foundational::facade::FieldKey::new("usd".to_owned())
                                .expect("valid foundational field key"),
                        ),
                    ),
                )],
            ),
        ),
        (
            "steel-route-replayed-against-combined-meaning",
            "commit:steel-main",
            pricing_reference_source_with_conflicting_route_commit_items(
                "commit:steel-main",
                "patch:steel-main-conflicting-combined",
                vec![
                    BridgeCommittedPatchItem::with_target(
                        "component:steel",
                        crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                            worth_foundational::facade::AspectLocator::new(
                                worth_foundational::facade::LocatorAuthority::Authoritative,
                                worth_foundational::facade::AspectKey::new("cost")
                                    .expect("valid bridge patch aspect key"),
                            ),
                            worth_foundational::facade::CanonicalFieldPath::single(
                                worth_foundational::facade::FieldKey::new("usd".to_owned())
                                    .expect("valid foundational field key"),
                            ),
                        ),
                    ),
                    BridgeCommittedPatchItem::with_target(
                        "component:rubber",
                        crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                            worth_foundational::facade::AspectLocator::new(
                                worth_foundational::facade::LocatorAuthority::Authoritative,
                                worth_foundational::facade::AspectKey::new("cost")
                                    .expect("valid bridge patch aspect key"),
                            ),
                            worth_foundational::facade::CanonicalFieldPath::single(
                                worth_foundational::facade::FieldKey::new("usd".to_owned())
                                    .expect("valid foundational field key"),
                            ),
                        ),
                    ),
                ],
            ),
        ),
    ] {
        let original_runtime = build_pricing_runtime(
            pricing_reference_source(),
            RecordingSignalBridgeSink::default(),
        );
        original_runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture(clean_commit),
                execution,
            )
            .unwrap_or_else(|_| panic!("{label} should route canonically before replay attack"));
        let canonical_record = original_runtime
            .diagnostics()
            .last_canonical_route_record()
            .unwrap_or_else(|| panic!("{label} should retain a canonical route record"));

        let restarted_runtime =
            build_pricing_runtime(mutated_source, RecordingSignalBridgeSink::default());
        let error = restarted_runtime
            .replay_canonical_record(&canonical_record, execution)
            .err()
            .unwrap_or_else(|| panic!("{label} should fail closed under non-commuting replay"));

        assert_eq!(
            error.kind(),
            BridgeReplayErrorKind::RouteMismatch,
            "{label} should classify as a replay route mismatch"
        );

        let failure_record = restarted_runtime
            .diagnostics()
            .last_failure_record()
            .unwrap_or_else(|| panic!("{label} should retain a failure record"));
        assert_eq!(
            failure_record.counters().route_replay_mismatch_count(),
            1,
            "{label} should increment replay mismatch exactly once"
        );
    }
}

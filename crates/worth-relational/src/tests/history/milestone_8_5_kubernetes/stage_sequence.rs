use super::*;

pub(super) fn run_kubernetes_style_certification(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> KubernetesIntentCertificationBundle {
    let medium = crate::tests::support::TestDurableMedium::new();
    let recovered_root = medium.clone();
    let runtime = persisted_strategy_runtime(medium);
    let entity = create_entity(execution, &runtime, "kube-service");
    let controller_branch = create_branch_from_main(&runtime, "kube-controller");
    let main_branch = BranchId("main".to_string());

    let broad_intent_commit = execute_strategy_commit(
        execution,
        &runtime,
        IntentReconciliationInput {
            entity_id: entity,
            desired_aspect_fields: strategy_name_and_replicas_patch("svc-v1", 3),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let first_converge_commit = execute_strategy_commit(
        execution,
        &runtime,
        ReplicaConvergenceInput {
            entity_id: entity,
            desired_replicas: 7,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(controller_branch.clone()),
    );
    let overlap_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_strategy_conflict(&overlap_planning, entity, "overlap");
    let overlap_conflict = planning_evidence(&overlap_planning);
    let overlap_main_head = runtime
        .history()
        .branch_head(&main_branch)
        .expect("overlap main head");
    let overlap_controller_head = runtime
        .history()
        .branch_head(&controller_branch)
        .expect("overlap controller head");
    let runtime = recover_stage(execution, &runtime, recovered_root.clone());
    let recovered_overlap_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_strategy_conflict(&recovered_overlap_planning, entity, "recovered overlap");
    assert_eq!(
        planning_evidence(&recovered_overlap_planning),
        overlap_conflict
    );

    let _narrowed_intent_commit = execute_strategy_commit(
        execution,
        &runtime,
        IntentReconciliationInput {
            entity_id: entity,
            desired_aspect_fields: crate::transactions::data::AspectFieldPatch::from_locator(
                crate::transactions::data::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name")
                        .expect("valid test aspect key"),
                    worth_foundational::facade::FieldKey::new("name")
                        .expect("valid test field key"),
                ),
                worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw("svc-v2".to_string()),
                ),
            ),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let idempotent_converge_commit = execute_strategy_commit(
        execution,
        &runtime,
        ReplicaConvergenceInput {
            entity_id: entity,
            desired_replicas: 7,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(controller_branch.clone()),
    );
    assert_eq!(
        idempotent_converge_commit
            .change_summary()
            .expect("idempotent change summary")
            .changed_record_count,
        0
    );
    let narrowed_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_non_strategy_conflict(&narrowed_planning, entity, "narrowed");
    let narrowed_non_conflict = planning_evidence(&narrowed_planning);
    let narrowed_main_head = runtime
        .history()
        .branch_head(&main_branch)
        .expect("narrowed main head");
    let narrowed_controller_head = runtime
        .history()
        .branch_head(&controller_branch)
        .expect("narrowed controller head");
    let runtime = recover_stage(execution, &runtime, recovered_root.clone());
    let recovered_narrowed_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_non_strategy_conflict(&recovered_narrowed_planning, entity, "recovered narrowed");
    assert_eq!(
        planning_evidence(&recovered_narrowed_planning),
        narrowed_non_conflict
    );

    let rebroadened_intent_commit = execute_strategy_commit(
        execution,
        &runtime,
        IntentReconciliationInput {
            entity_id: entity,
            desired_aspect_fields: strategy_name_and_replicas_patch("svc-v2", 9),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let rebroadened_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_strategy_conflict(&rebroadened_planning, entity, "rebroadened");
    let rebroadened_conflict = planning_evidence(&rebroadened_planning);
    let rebroadened_main_head = runtime
        .history()
        .branch_head(&main_branch)
        .expect("rebroadened main head");
    let rebroadened_controller_head = runtime
        .history()
        .branch_head(&controller_branch)
        .expect("rebroadened controller head");
    let rebroadened_intent_replay = replay_commit(
        execution,
        &runtime,
        rebroadened_intent_commit.commit.commit_id,
        main_branch.clone(),
    );
    assert!(rebroadened_intent_replay.failure.is_none());
    assert!(rebroadened_intent_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    let runtime = recover_stage(execution, &runtime, recovered_root.clone());
    let recovered_rebroadened_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_strategy_conflict(
        &recovered_rebroadened_planning,
        entity,
        "recovered rebroadened",
    );
    assert_eq!(
        planning_evidence(&recovered_rebroadened_planning),
        rebroadened_conflict
    );
    let recovered_rebroadened_intent_replay = replay_commit(
        execution,
        &runtime,
        rebroadened_intent_commit.commit.commit_id,
        main_branch.clone(),
    );
    assert_eq!(
        recovered_rebroadened_intent_replay,
        rebroadened_intent_replay
    );

    let revalidation_commit = execute_strategy_commit(
        execution,
        &runtime,
        ReplicaConvergenceInput {
            entity_id: entity,
            desired_replicas: 9,
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        Some(controller_branch.clone()),
    );
    assert_eq!(
        revalidation_commit
            .change_summary()
            .expect("revalidation change summary")
            .changed_record_count,
        1
    );
    let revalidated_planning = planning_for(
        execution,
        &runtime,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_converged_strategy_overlap(
        &revalidated_planning,
        entity,
        "revalidated converged strategy overlap",
    );
    let revalidated_converged_overlap = planning_evidence(&revalidated_planning);
    let revalidation_replay = replay_commit(
        execution,
        &runtime,
        revalidation_commit.commit.commit_id,
        controller_branch.clone(),
    );
    assert_strategy_replay_clean(&revalidation_replay, "revalidation converge");

    let broad_intent_replay = replay_commit(
        execution,
        &runtime,
        broad_intent_commit.commit.commit_id,
        main_branch.clone(),
    );
    let first_converge_replay = replay_commit(
        execution,
        &runtime,
        first_converge_commit.commit.commit_id,
        controller_branch.clone(),
    );
    let rebroadened_intent_replay = replay_commit(
        execution,
        &runtime,
        rebroadened_intent_commit.commit.commit_id,
        main_branch.clone(),
    );
    assert_strategy_replay_clean(&broad_intent_replay, "broad intent");
    assert_strategy_replay_clean(&first_converge_replay, "first converge");
    assert_strategy_replay_clean(&rebroadened_intent_replay, "rebroadened intent");

    let visible_truth = KubernetesBranchVisibleTruthEvidence {
        main: visible_truth_for_branch(execution, &runtime, &main_branch, entity),
        controller: visible_truth_for_branch(execution, &runtime, &controller_branch, entity),
    };
    let live_bundle = KubernetesIntentCertificationBundle {
        overlap_conflict,
        narrowed_non_conflict,
        rebroadened_conflict,
        revalidated_converged_overlap,
        revalidation_noop: KubernetesNoopEvidence {
            strategy_artifacts: revalidation_commit
                .publication()
                .strategy_artifacts
                .as_ref()
                .expect("revalidation strategy artifacts")
                .clone(),
            changed_record_count: revalidation_commit
                .change_summary()
                .expect("revalidation change summary")
                .changed_record_count,
            patch_record_count: revalidation_commit
                .publication()
                .envelope
                .patch
                .authoritative_record_patches
                .len(),
        },
        broad_intent_replay,
        first_converge_replay,
        rebroadened_intent_replay,
        revalidation_noop_replay: revalidation_replay,
        branch_heads: KubernetesBranchHeadEvidence {
            main: runtime.history().branch_head(&main_branch),
            controller: runtime.history().branch_head(&controller_branch),
        },
        visible_truth,
    };

    final_recovery::certify_final_recovery(
        execution,
        final_recovery::FinalRecoveryCertificationInput {
            runtime,
            recovered_root,
            entity,
            main_branch,
            controller_branch,
            overlap_main_head,
            overlap_controller_head,
            narrowed_main_head,
            narrowed_controller_head,
            rebroadened_main_head,
            rebroadened_controller_head,
            broad_intent_commit,
            first_converge_commit,
            rebroadened_intent_commit,
            revalidation_commit,
            live_bundle,
        },
    )
}

//! End-to-end courts for Store M4.3f (`P0-A` paged naming & group state +
//! issue `#440` non-head persistent group survival + `C1 -> C2 -> C3` carry
//! and post-ack working-set eviction).
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    RelationalDurableCheckpointSource, RelationalDurableColdSource,
    RelationalDurableRecoverySource,
};
use worth_relational::facade::lineage::{PersistentNameKey, PersistentNameSelection};
use worth_testkit_relational::cold_recovery::{
    capture_external_carrying, capture_external_checkpoint, immutable_source, update_entity,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    commit_page_reads, create_named_entities, durable_named_runtime, lineage_page_reads,
    naming_page_reads, naming_pages, observe_branch, publish_group_partition, reopen_named,
    GROUP_SCHEME,
};

#[test]
fn issue_440_non_head_persistent_groups_survive_cold_reopen_tail_replay_and_c1_to_c2() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    // Commit 1: create named entities e1, e2.
    let (_, entities_1, names_1) =
        create_named_entities(request, &producer, &main, &["g-e1".into(), "g-e2".into()]);
    // Commit 2 (non-head!): publish persistent group G1 = {e1, e2}.
    let _c2 = publish_group_partition(
        request,
        &producer,
        &main,
        entities_1[0],
        "group-c2",
        vec![vec![
            (names_1[0], AggregateMemberRole::Anchor),
            (names_1[1], AggregateMemberRole::Interior),
        ]],
    );
    let basis_c2 = observe_branch(&producer, &main);
    let group_key = producer
        .with_persistent_names_at_observation(request, &basis_c2.observation(), |view| {
            view.group_of_member(GROUP_SCHEME, &names_1[0])
                .expect("group born at C2")
        })
        .unwrap();

    // Commit 3 & 4: later commits that do NOT touch G1, making C2 strictly non-head.
    let (_, entities_3, names_3) =
        create_named_entities(request, &producer, &main, &["g-e3".into()]);
    update_entity(request, &producer, entities_3[0], "later-unrelated-head");

    let (ck1, _) = capture_external_checkpoint(request, &producer);
    let ck1_source = immutable_source(&ck1, &medium);
    let covered_pos = ck1.covered_position().unwrap();
    let covered_trans = ck1.covered_branch_transitions().unwrap();

    // Tail commit after CK1: expand non-head group G1 to include e3.
    let _c5_tail = publish_group_partition(
        request,
        &producer,
        &main,
        entities_3[0],
        "group-tail-expand",
        vec![vec![
            (names_1[0], AggregateMemberRole::Anchor),
            (names_1[1], AggregateMemberRole::Interior),
            (names_3[0], AggregateMemberRole::Interior),
        ]],
    );

    // Check B: Cold reopen from CK1 + tail replay of C5 preserves non-head G1 and applies C5.
    let tail_commits: Vec<_> = medium
        .committed_artifacts()
        .unwrap()
        .into_iter()
        .filter(|artifact| artifact.position() > covered_pos)
        .collect();
    assert_eq!(tail_commits.len(), 1);
    let tail_pos = tail_commits[0].position();
    let mut replayed = durable_named_runtime(&medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            covered_pos,
            covered_trans,
            ck1.clone(),
        )),
        tail_commits,
        Vec::new(),
        ck1_source.clone(),
    );
    let plan = replayed
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    replayed
        .durability_recovery()
        .recover(request, plan)
        .expect("tail replay referencing non-head group from C1 must succeed");
    let replayed_head = observe_branch(&replayed, &main);
    replayed
        .with_persistent_names_at_observation(request, &replayed_head.observation(), |view| {
            assert_eq!(
                view.group_of_member(GROUP_SCHEME, &names_3[0]),
                Some(group_key)
            );
            assert_eq!(view.group_members(group_key).unwrap().len(), 3);
        })
        .unwrap();

    // Check A: Cold reopen from CK1 alone (zero tail) preserves non-head G1 with zero page reads,
    // then mutates G1 after reopen and captures/reopens CK2.
    medium.drop_artifact(tail_pos);
    let ck1_dir = super::selected_directory::read_native(&ck1);
    let ck1_fresh_source = immutable_source(&ck1, &medium);
    let reopened = reopen_named(
        request,
        ck1.clone(),
        ck1_fresh_source.clone(),
        &medium,
        u64::MAX / 4,
    );
    assert_eq!(naming_page_reads(&ck1_fresh_source), 0);
    assert_eq!(commit_page_reads(&ck1_fresh_source), 0);
    assert_eq!(lineage_page_reads(&ck1_fresh_source, &ck1_dir), 0);

    let head = observe_branch(&reopened, &main);
    reopened
        .with_persistent_names_at_observation(request, &head.observation(), |view| {
            assert_eq!(
                view.group_of_member(GROUP_SCHEME, &names_1[0]),
                Some(group_key)
            );
            assert_eq!(
                view.group_of_member(GROUP_SCHEME, &names_1[1]),
                Some(group_key)
            );
            let members = view.group_members(group_key).expect("G1 members survive");
            assert_eq!(members.len(), 2);
            assert!(members.contains(&names_1[0]));
            assert!(members.contains(&names_1[1]));
            assert!(view
                .resolve(group_key, &PersistentNameSelection::Exact)
                .result
                .is_ok());
        })
        .unwrap();

    // Update G1 after cold reopen and capture C2 -> reopen C2 -> G1 persists with exact identity.
    publish_group_partition(
        request,
        &reopened,
        &main,
        entities_3[0],
        "post-reopen-expand",
        vec![vec![
            (names_1[0], AggregateMemberRole::Anchor),
            (names_1[1], AggregateMemberRole::Interior),
            (names_3[0], AggregateMemberRole::Interior),
        ]],
    );
    let (ck2, ck2_source) = capture_external_carrying(request, &reopened, &ck1_fresh_source);
    let reopened_ck2 = reopen_named(request, ck2, ck2_source, &medium, u64::MAX / 4);
    let head2 = observe_branch(&reopened_ck2, &main);
    reopened_ck2
        .with_persistent_names_at_observation(request, &head2.observation(), |view| {
            assert_eq!(
                view.group_of_member(GROUP_SCHEME, &names_3[0]),
                Some(group_key),
                "G1 identity preserved across C1 -> post-reopen edit -> C2 reopen"
            );
            assert_eq!(view.group_members(group_key).unwrap().len(), 3);
        })
        .unwrap();
}

#[test]
fn cold_reopen_reads_zero_naming_pages_and_c1_c2_c3_carries_and_evicts() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let mut all_names = Vec::new();
    for batch_idx in 0..30 {
        let keys: Vec<String> = (0..10)
            .map(|j| format!("ent-{batch_idx:03}-{j:02}"))
            .collect();
        let (_, entities, names) = create_named_entities(request, &producer, &main, &keys);
        publish_group_partition(
            request,
            &producer,
            &main,
            entities[0],
            &format!("grp-{batch_idx:03}"),
            vec![vec![
                (names[0], AggregateMemberRole::Anchor),
                (names[1], AggregateMemberRole::Interior),
            ]],
        );
        all_names.extend(names);
    }

    let (c1, c1_source) = capture_external_checkpoint(request, &producer);
    let c1_pages = naming_pages(&c1_source);
    assert!(
        c1_pages.len() >= 6,
        "fixture must span multiple naming pages, got {}",
        c1_pages.len()
    );
    let c1_dir = super::selected_directory::read_native(&c1);
    let tight_budget = 64 * 1024;
    let runtime = reopen_named(
        request,
        c1.clone(),
        c1_source.clone(),
        &medium,
        tight_budget,
    );
    assert_eq!(
        naming_page_reads(&c1_source),
        0,
        "reopen reads zero naming pages"
    );
    assert_eq!(
        commit_page_reads(&c1_source),
        0,
        "reopen reads zero commit pages"
    );
    assert_eq!(
        lineage_page_reads(&c1_source, &c1_dir),
        0,
        "reopen reads zero lineage node/event pages"
    );

    // Point lookup faults strictly fewer than all naming pages.
    let head = observe_branch(&runtime, &main);
    runtime
        .with_persistent_names_at_observation(request, &head.observation(), |view| {
            let key = PersistentNameKey::Entity(all_names[0]);
            assert!(view
                .resolve(key, &PersistentNameSelection::Exact)
                .result
                .is_ok());
            assert!(view.group_of_member(GROUP_SCHEME, &all_names[0]).is_some());
        })
        .unwrap();
    let after_point = naming_page_reads(&c1_source);
    assert!(
        after_point > 0 && after_point < c1_pages.len(),
        "point lookup faults only touched pages: {after_point} of {}",
        c1_pages.len()
    );

    // Reopen from `medium` with tight historical budget for real C1 -> ack C2 -> C3.
    medium.file_checkpoint(c1.clone());
    let mut runtime = durable_named_runtime(&medium, tight_budget);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();

    // Commit a single named entity after reopen and capture C2 carrying C1's members.
    let (_, _, _) = create_named_entities(request, &runtime, &main, &["tail-one".into()]);
    let current_members: Vec<_> = c1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let medium_source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let c2 = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: Some(medium_source.clone()),
            },
        )
        .unwrap();
    let carried_naming = c1_pages
        .iter()
        .filter(|locator| {
            c2.members().iter().any(|member| {
                matches!(member, RelationalCheckpointMember::Carried { locator: c } if c == *locator)
            })
        })
        .count();
    assert!(
        carried_naming >= 1,
        "unchanged naming pages must be carried into C2 (carried {carried_naming} of {})",
        c1_pages.len()
    );

    // Acknowledge C2 and traverse all names/groups under the tight historical budget.
    medium.file_checkpoint(c2.clone());
    request
        .run_native_with_checkpoint(|context| c2.acknowledge_selected(request, context))
        .unwrap()
        .0
        .unwrap();
    let head_after = observe_branch(&runtime, &main);
    runtime
        .with_persistent_names_at_observation(request, &head_after.observation(), |view| {
            for name in &all_names {
                let key = PersistentNameKey::Entity(*name);
                assert!(view
                    .resolve(key, &PersistentNameSelection::Exact)
                    .result
                    .is_ok());
            }
        })
        .unwrap();
    let pressure = runtime.residency_pressure().unwrap();
    assert!(
        pressure.historical_resident_bytes <= tight_budget,
        "post-ack naming store must obey historical budget: {pressure:?}"
    );
    assert!(
        pressure.evictions_completed > 0,
        "post-ack naming store must evict under tight historical budget: {pressure:?}"
    );

    // C3 with zero new commits carries every C2 naming page.
    let c2_dir = super::selected_directory::read_native(&c2);
    let c2_naming_pages: Vec<_> = c2_dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .chain(&c2_dir.naming.group_log)
        .map(|p| p.member)
        .collect();
    let c2_members: Vec<_> = c2
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let c3 = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &c2_members,
                source: Some(medium_source),
            },
        )
        .unwrap();
    assert!(c2_naming_pages.iter().all(|locator| {
        c3.members().iter().any(|member| {
            matches!(member, RelationalCheckpointMember::Carried { locator: c } if *c == *locator)
        })
    }));
}

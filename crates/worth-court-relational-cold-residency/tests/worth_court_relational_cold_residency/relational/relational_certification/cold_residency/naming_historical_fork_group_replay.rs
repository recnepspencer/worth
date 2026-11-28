//! Cold-residency courts for historical-fork single-read lineage preparation
//! and per-branch `GROUP_LOG` window replay:
//! - Cold historical forks carry reconstructed lineage through durable
//!   publication so post-durable installation performs zero cold reads.
//! - `rebuild_historical_group_state` routes through the per-branch `GROUP_LOG`
//!   window index so unavailable sibling `GROUP_LOG` pages are never opened.

use std::collections::BTreeMap;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{PersistentNameKey, PersistentNameSelection};
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, observe_branch, publish_group_partition,
    reopen_named, GROUP_SCHEME,
};
use super::naming_branch_root_and_dissolve::fork_retained_at;

#[test]
fn historical_fork_reconstructs_groups_without_opening_sibling_group_log_pages() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["r15-g-a".into(), "r15-g-b".into(), "r15-g-c".into()],
    );
    let _ = publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "r15-main-g1",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Interior),
        ]],
    );
    let g1 = producer
        .with_persistent_names_at_observation(
            request,
            &observe_branch(&producer, &main).observation(),
            |v| v.group_of_member(GROUP_SCHEME, &names[0]).unwrap(),
        )
        .unwrap();

    let sibling = BranchId("r15-sibling".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &sibling,
    )
    .unwrap();
    let _ = publish_group_partition(
        request,
        &producer,
        &sibling,
        entities[0],
        "r15-sibling-g",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[2], AggregateMemberRole::Interior),
        ]],
    );

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let primary_pages: Vec<_> = super::selected_directory::read_native(&ck)
        .naming
        .group_log
        .into_iter()
        .filter(|p| p.first.first() == Some(&14))
        .collect();
    assert_eq!(
        primary_pages.len(),
        2,
        "expected separate single-branch GROUP_LOG pages for main and sibling"
    );
    let main_group_log_page = primary_pages[0].member;
    let sibling_group_log_page = primary_pages[1].member;

    let source = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck, source.clone(), &medium, u64::MAX / 4);
    let basis_before_c2 = observe_branch(&reopened, &main);

    // Advance main past basis_before_c2 so forking at basis_before_c2 must
    // reconstruct main's historical group state from cold GROUP_LOG pages.
    let _ = publish_group_partition(
        request,
        &reopened,
        &main,
        entities[0],
        "r15-main-g2",
        vec![vec![
            (names[1], AggregateMemberRole::Anchor),
            (names[2], AggregateMemberRole::Interior),
        ]],
    );

    source.failures.lock().unwrap().insert(
        ColdRead::Member(sibling_group_log_page),
        Cause::MissingMember,
    );
    let reads_before = source.naming_page_reads.lock().unwrap().len();

    let hist_child = BranchId("r15-hist-child".into());
    fork_retained_at(request, &reopened, &basis_before_c2, &hist_child)
        .expect("historical fork must reconstruct main groups without opening sibling GROUP_LOG");

    let fork_reads = &source.naming_page_reads.lock().unwrap()[reads_before..];
    let main_reads = fork_reads
        .iter()
        .filter(|m| **m == main_group_log_page)
        .count();
    let sibling_reads = fork_reads
        .iter()
        .filter(|m| **m == sibling_group_log_page)
        .count();
    assert_eq!(
        main_reads, 1,
        "main's GROUP_LOG page must be read exactly once during historical fork preparation"
    );
    assert_eq!(
        sibling_reads, 0,
        "sibling GROUP_LOG page must not be opened when rebuilding main's historical group state"
    );
    let child_obs = observe_branch(&reopened, &hist_child).observation();
    reopened
        .with_persistent_names_at_observation(request, &child_obs, |v| {
            assert_eq!(v.group_of_member(GROUP_SCHEME, &names[0]), Some(g1));
            assert_eq!(v.group_of_member(GROUP_SCHEME, &names[1]), Some(g1));
            assert_eq!(v.group_of_member(GROUP_SCHEME, &names[2]), None);
        })
        .unwrap();
}

#[test]
fn historical_fork_installs_prepared_lineage_under_tight_cache_without_second_read() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    // Create enough named entities across batches to span multiple cold naming
    // index pages on `main`, plus a live group partition.
    let mut all_entities = Vec::new();
    let mut all_names = Vec::new();
    for b in 0..24usize {
        let keys: Vec<String> = (0..16).map(|i| format!("r15-e-{b:02}-{i:02}")).collect();
        let (_, e, n) = create_named_entities(request, &producer, &main, &keys);
        all_entities.extend(e);
        all_names.extend(n);
    }
    let _ = publish_group_partition(
        request,
        &producer,
        &main,
        all_entities[0],
        "r15-tight-g1",
        vec![vec![
            (all_names[0], AggregateMemberRole::Anchor),
            (all_names[1], AggregateMemberRole::Interior),
        ]],
    );
    let g1 = producer
        .with_persistent_names_at_observation(
            request,
            &observe_branch(&producer, &main).observation(),
            |v| v.group_of_member(GROUP_SCHEME, &all_names[0]).unwrap(),
        )
        .unwrap();

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let dir = super::selected_directory::read_native(&ck);
    let index_name_pages: Vec<_> = dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .filter(|p| p.first.first() == Some(&6) && p.last.first() == Some(&6))
        .map(|p| p.member)
        .collect();
    let group_log_pages: Vec<_> = dir
        .naming
        .group_log
        .iter()
        .filter(|p| p.first.first() == Some(&14))
        .map(|p| p.member)
        .collect();
    assert!(
        !index_name_pages.is_empty() && !group_log_pages.is_empty(),
        "fixture must emit pure cold INDEX_NAME and GROUP_LOG pages"
    );

    // Reopen with a tight historical working-set budget (~1 full naming page) so
    // early pages read during fork preparation are evicted before preparation finishes.
    let source = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck, source.clone(), &medium, 64 * 1024);
    let basis_c1 = observe_branch(&reopened, &main);

    // Advance main with a new named entity and group change so forking at
    // `basis_c1` requires historical slicing of index, naming basis, and groups.
    let (_, _, post_names) = create_named_entities(request, &reopened, &main, &["r15-post".into()]);
    let _ = publish_group_partition(
        request,
        &reopened,
        &main,
        all_entities[0],
        "r15-tight-g2",
        vec![vec![
            (all_names[1], AggregateMemberRole::Anchor),
            (all_names[2], AggregateMemberRole::Interior),
        ]],
    );

    let reads_before = source.naming_page_reads.lock().unwrap().len();
    let hist_child = BranchId("r15-tight-child".into());
    fork_retained_at(request, &reopened, &basis_c1, &hist_child)
        .expect("historical fork under tight cache must succeed");

    // Verify no INDEX_NAME or GROUP_LOG page was faulted a second time during
    // post-durable installation: even though the tight 64 KiB cache evicts
    // earlier pages during preparation, each page is read exactly once.
    let mut counts: BTreeMap<_, usize> = BTreeMap::new();
    for member in &source.naming_page_reads.lock().unwrap()[reads_before..] {
        *counts.entry(*member).or_default() += 1;
    }
    for member in index_name_pages.iter().chain(&group_log_pages) {
        assert_eq!(
            counts.get(member).copied().unwrap_or(0),
            1,
            "cold naming page {member:?} must be read exactly once during historical fork; installation must perform zero cold reads"
        );
    }

    // Verify child installed intact historical naming, index, and group state
    // and can continue committing named entities cleanly.
    let child_obs = observe_branch(&reopened, &hist_child).observation();
    reopened
        .with_persistent_names_at_observation(request, &child_obs, |v| {
            assert!(v
                .resolve(
                    PersistentNameKey::Entity(all_names[0]),
                    &PersistentNameSelection::Exact,
                )
                .result
                .is_ok());
            assert!(v
                .resolve(
                    PersistentNameKey::Entity(post_names[0]),
                    &PersistentNameSelection::Exact,
                )
                .result
                .is_err());
            assert_eq!(v.group_of_member(GROUP_SCHEME, &all_names[0]), Some(g1));
        })
        .unwrap();
    let (_, _, child_new) =
        create_named_entities(request, &reopened, &hist_child, &["r15-child-new".into()]);
    let child_after = observe_branch(&reopened, &hist_child).observation();
    reopened
        .with_persistent_names_at_observation(request, &child_after, |v| {
            assert!(v
                .resolve(
                    PersistentNameKey::Entity(child_new[0]),
                    &PersistentNameSelection::Exact,
                )
                .result
                .is_ok());
        })
        .unwrap();
}

#[test]
fn deep_fork_chain_historical_group_replay_preserves_checkpoint_order_via_min_heap() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let mut current = producer.main_branch_identity().branch_id().clone();

    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &current,
        &["r16-a".into(), "r16-b".into(), "r16-c".into()],
    );
    // Build a 10-ancestor chain where each branch publishes one group partition
    // (producing 10 distinct single-branch primary GROUP_LOG pages).
    for level in 0..10usize {
        let pair = if level % 2 == 0 {
            vec![
                (names[0], AggregateMemberRole::Anchor),
                (names[1], AggregateMemberRole::Interior),
            ]
        } else {
            vec![
                (names[0], AggregateMemberRole::Anchor),
                (names[2], AggregateMemberRole::Interior),
            ]
        };
        let _ = publish_group_partition(
            request,
            &producer,
            &current,
            entities[0],
            &format!("r16-lvl-{level}"),
            vec![pair],
        );
        if level + 1 < 10 {
            let next = BranchId(format!("r16-chain-{}", level + 1));
            fork_retained_at(
                request,
                &producer,
                &observe_branch(&producer, &current),
                &next,
            )
            .unwrap();
            current = next;
        }
    }
    let expected_groups = producer
        .with_persistent_names_at_observation(
            request,
            &observe_branch(&producer, &current).observation(),
            |v| {
                (
                    v.group_of_member(GROUP_SCHEME, &names[0]),
                    v.group_of_member(GROUP_SCHEME, &names[1]),
                    v.group_of_member(GROUP_SCHEME, &names[2]),
                )
            },
        )
        .unwrap();
    assert!(expected_groups.0.is_some() && expected_groups.0 == expected_groups.2);

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let group_log_pages: Vec<_> = super::selected_directory::read_native(&ck)
        .naming
        .group_log
        .into_iter()
        .filter(|p| p.first.first() == Some(&14))
        .map(|p| p.member)
        .collect();
    assert_eq!(group_log_pages.len(), 10);

    let source = immutable_source(&ck, &medium);
    let reopened = reopen_named(request, ck, source.clone(), &medium, u64::MAX / 4);
    let basis_before = observe_branch(&reopened, &current);
    let _ = publish_group_partition(
        request,
        &reopened,
        &current,
        entities[0],
        "r16-post",
        vec![vec![
            (names[1], AggregateMemberRole::Anchor),
            (names[2], AggregateMemberRole::Interior),
        ]],
    );

    let reads_before = source.naming_page_reads.lock().unwrap().len();
    let child = BranchId("r16-deep-hist-child".into());
    fork_retained_at(request, &reopened, &basis_before, &child).unwrap();
    let mut counts: BTreeMap<_, usize> = BTreeMap::new();
    for m in &source.naming_page_reads.lock().unwrap()[reads_before..] {
        *counts.entry(*m).or_default() += 1;
    }
    for page in &group_log_pages {
        assert_eq!(counts.get(page).copied().unwrap_or(0), 1);
    }
    let child_obs = observe_branch(&reopened, &child).observation();
    let child_groups = reopened
        .with_persistent_names_at_observation(request, &child_obs, |v| {
            (
                v.group_of_member(GROUP_SCHEME, &names[0]),
                v.group_of_member(GROUP_SCHEME, &names[1]),
                v.group_of_member(GROUP_SCHEME, &names[2]),
            )
        })
        .unwrap();
    assert_eq!(child_groups, expected_groups);
}

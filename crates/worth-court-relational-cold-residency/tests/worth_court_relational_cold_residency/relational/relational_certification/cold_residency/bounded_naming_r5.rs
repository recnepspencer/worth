//! Round 5 hardening courts for Store M4.3f (`[P1-1]` and `[P1-2]`):
//! - `[P1-1]` Overlapping/stale ACKs serialize directory selection and naming
//!   consolidation so an older capture can never rebind live overlays backward.
//! - `[P1-2]` Tampered `CHK24` stripping primary `GROUP_LOG` page descriptors
//!   while keeping `VALUE_CHUNK` descriptors and `group_log_through` refuses
//!   with `CorruptCheckpoint`.

use std::sync::{Arc, Barrier};
use std::thread;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalCheckpointAcknowledgementDenial,
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource, RelationalNativeCheckpoint,
    RelationalRecoveryStop,
};
use worth_testkit_relational::cold_recovery::capture_external_checkpoint;
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, observe_branch, publish_group_partition,
};
use super::bounded_naming_r3_ack::{capture_only, try_ack};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};

fn strip_primary_group_log_pages_keep_chunks(
    ck: &RelationalNativeCheckpoint,
) -> RelationalNativeCheckpoint {
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let pages = rows(field(naming, "group_log"));
    let mut kept: Vec<&[u8]> = Vec::new();
    let mut stripped_primary = 0usize;
    let mut kept_chunks = 0usize;
    for page in pages {
        let first: Vec<u8> = rmp_serde::from_slice(field(page, "first")).unwrap();
        if first[0] == 14 {
            stripped_primary += 1;
        } else {
            assert_eq!(first[0], 15, "expected VALUE_CHUNK tag 15");
            kept_chunks += 1;
            kept.push(page);
        }
    }
    assert!(stripped_primary > 0, "expected primary GROUP_LOG pages");
    assert!(
        kept_chunks > 0,
        "fixture must emit chunked VALUE_CHUNK pages in group_log"
    );
    let new_naming = replace_field(naming, "group_log", &array(&kept));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + new_body.len());
    bytes.extend_from_slice(CHECKPOINT_MAGIC);
    bytes.extend_from_slice(&new_body);
    RelationalNativeCheckpoint::from_untrusted_bytes(bytes)
}

#[test]
fn overlapping_and_stale_acks_never_rebind_live_naming_back_to_older_checkpoint() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = Arc::new(durable_named_runtime(&medium, u64::MAX / 4));
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, _, n1) = create_named_entities(request, &producer, &main, &["r5-c1".into()]);
    let ck1 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    let (_, _, n2) = create_named_entities(request, &producer, &main, &["r5-c2".into()]);
    let ck2 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck2.clone());

    let (_, _, n3) = create_named_entities(request, &producer, &main, &["r5-c3".into()]);
    let ck3 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck3.clone());

    // Race C2 and C3 ACKs concurrently across two threads: regardless of which
    // wins first, once C3 is acknowledged, C2 must never roll live naming back.
    let ck2_clone = ck2.clone();
    let ck3_clone = ck3.clone();
    let t2 = thread::spawn(move || {
        let h = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let req = ExecutionRequest::serial(&h);
        try_ack(&req, &ck2_clone)
    });
    let t3 = thread::spawn(move || {
        let h = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let req = ExecutionRequest::serial(&h);
        try_ack(&req, &ck3_clone)
    });
    let _ = t2.join().unwrap();
    let r3 = t3.join().unwrap();
    if r3.is_err() {
        try_ack(request, &ck3).unwrap();
    }
    // Re-attempting stale C2 after C3 is acknowledged must refuse and leave C3 intact.
    assert!(try_ack(request, &ck2).is_err());

    let live = producer.branch_live_persistent_names(&main).unwrap();
    assert!(live.contains_key(&n1[0]));
    assert!(live.contains_key(&n2[0]));
    assert!(
        live.contains_key(&n3[0]),
        "C3 name must remain live after overlapping/stale C2 ACK"
    );
}

#[test]
fn tampered_chk24_stripping_primary_group_log_pages_keeping_chunks_refuses_corrupt_checkpoint() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    // Create enough named members so the encoded GROUP_LOG transition frame exceeds
    // MAX_INLINE_VALUE_BYTES (24 KiB) and emits both GROUP_LOG (14) and VALUE_CHUNK (15).
    let labels: Vec<String> = (0..900)
        .map(|i| format!("r5-chunked-member-{i:04}"))
        .collect();
    let (_, entities, names) = create_named_entities(request, &producer, &main, &labels);
    let members: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            (
                name,
                if i == 0 {
                    AggregateMemberRole::Anchor
                } else {
                    AggregateMemberRole::Ordinary
                },
            )
        })
        .collect();
    publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "r5-big-group",
        vec![members],
    );

    let (ck, source) = capture_external_checkpoint(request, &producer);
    let tampered = strip_primary_group_log_pages_keep_chunks(&ck);
    let mut target = durable_named_runtime(&medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            tampered,
        )),
        Vec::new(),
        Vec::new(),
        source,
    );
    let err = match target.durability().recovery_plan_from_source(
        request,
        input,
        RecoveryVerificationMode::NormalRecoveryVerification,
    ) {
        Err(err) => err,
        Ok(plan) => target
            .durability_recovery()
            .recover(request, plan)
            .expect_err("stripping primary GROUP_LOG pages must refuse CorruptCheckpoint"),
    };
    assert!(
        matches!(
            err,
            RelationalRecoveryStop::Durability(ref d)
                if d.class == RecoveryFailureClass::CorruptCheckpoint
        ),
        "expected CorruptCheckpoint, got {err:?}"
    );
}

fn drop_first_primary_group_log_page_keep_last(
    ck: &RelationalNativeCheckpoint,
) -> RelationalNativeCheckpoint {
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let pages = rows(field(naming, "group_log"));
    let mut kept: Vec<&[u8]> = Vec::new();
    let mut primary_seen = 0usize;
    for page in pages {
        let first: Vec<u8> = rmp_serde::from_slice(field(page, "first")).unwrap();
        if first[0] == 14 {
            primary_seen += 1;
            if primary_seen == 1 {
                continue;
            }
        }
        kept.push(page);
    }
    assert!(
        primary_seen >= 2,
        "fixture must emit at least two primary GROUP_LOG pages (saw {primary_seen})"
    );
    let new_naming = replace_field(naming, "group_log", &array(&kept));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + new_body.len());
    bytes.extend_from_slice(CHECKPOINT_MAGIC);
    bytes.extend_from_slice(&new_body);
    RelationalNativeCheckpoint::from_untrusted_bytes(bytes)
}

#[test]
fn ack_returns_busy_without_deadlock_while_naming_read_callback_observes_cold_stats() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = Arc::new(durable_named_runtime(&medium, u64::MAX / 4));
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, _, n1) = create_named_entities(request, &producer, &main, &["r6-c1".into()]);
    let ck1 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    let (_, _, n2) = create_named_entities(request, &producer, &main, &["r6-c2".into()]);
    let ck2 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck2.clone());

    let entered_cb = Arc::new(Barrier::new(2));
    let ack_done = Arc::new(Barrier::new(2));
    let ck2_bg = ck2.clone();
    let entered_bg = Arc::clone(&entered_cb);
    let done_bg = Arc::clone(&ack_done);
    let ack_thread = thread::spawn(move || {
        let h = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let req = ExecutionRequest::serial(&h);
        entered_bg.wait();
        let result = try_ack(&req, &ck2_bg);
        done_bg.wait();
        result
    });

    let after = observe_branch(&producer, &main);
    let (obs_before, obs_after) = producer
        .with_persistent_names_at_observation(request, &after.observation(), |_view| {
            entered_cb.wait();
            let before = producer.cold_naming_observation();
            ack_done.wait();
            let after = producer.cold_naming_observation();
            (before.is_some(), after.is_some())
        })
        .unwrap();
    assert!(obs_before && obs_after);

    let ack_result = ack_thread.join().unwrap();
    assert!(
        matches!(
            ack_result,
            Err(RelationalCheckpointAcknowledgementDenial::Busy)
        ),
        "ACK must yield Busy instead of blocking on lineage write lock, got {ack_result:?}"
    );

    // Once the read callback drops the lineage read lock, retrying ACK succeeds.
    try_ack(request, &ck2).unwrap();
    let live = producer.branch_live_persistent_names(&main).unwrap();
    assert!(live.contains_key(&n1[0]));
    assert!(live.contains_key(&n2[0]));
}

#[test]
fn tampered_chk24_dropping_earlier_primary_group_log_page_refuses_corrupt_checkpoint() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    // Rotating the Anchor across 4 commits dissolves the prior group and emits a
    // fresh ~450-member inline creation frame (~15 KiB < 24 KiB chunking limit,
    // ~30 KiB worst-case row budget each), spilling across >= 2 primary GROUP_LOG pages.
    let labels: Vec<String> = (0..450)
        .map(|i| format!("r6-inline-member-{i:04}"))
        .collect();
    let (_, entities, names) = create_named_entities(request, &producer, &main, &labels);
    for step in 0..4usize {
        let members: Vec<_> = names
            .iter()
            .enumerate()
            .map(|(i, &name)| {
                (
                    name,
                    if i == step {
                        AggregateMemberRole::Anchor
                    } else {
                        AggregateMemberRole::Ordinary
                    },
                )
            })
            .collect();
        publish_group_partition(
            request,
            &producer,
            &main,
            entities[0],
            &format!("r6-page-{step}"),
            vec![members],
        );
    }

    let (ck, source) = capture_external_checkpoint(request, &producer);
    let tampered = drop_first_primary_group_log_page_keep_last(&ck);
    let mut target = durable_named_runtime(&medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            tampered,
        )),
        Vec::new(),
        Vec::new(),
        source,
    );
    let err = match target.durability().recovery_plan_from_source(
        request,
        input,
        RecoveryVerificationMode::NormalRecoveryVerification,
    ) {
        Err(err) => err,
        Ok(plan) => target
            .durability_recovery()
            .recover(request, plan)
            .expect_err("dropping an earlier primary GROUP_LOG page must refuse CorruptCheckpoint"),
    };
    assert!(
        matches!(
            err,
            RelationalRecoveryStop::Durability(ref d)
                if d.class == RecoveryFailureClass::CorruptCheckpoint
        ),
        "expected CorruptCheckpoint, got {err:?}"
    );
}

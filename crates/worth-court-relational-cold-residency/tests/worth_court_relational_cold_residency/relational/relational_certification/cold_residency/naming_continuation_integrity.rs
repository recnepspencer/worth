//! Cold-residency courts for long `BranchId` group-log digest keys and
//! `VALUE_CHUNK` continuation manifest integrity:
//! - Long valid `BranchId`s (e.g. 200-byte ASCII names) publish group
//!   transitions, checkpoint to `CHK24`, carry across C1->C2, reopen with zero
//!   naming page reads, and replay historical group state cleanly.
//! - Tampered `CHK24` dropping or reassigning `VALUE_CHUNK` continuation
//!   descriptors fails closed before install or on first fault.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalDurableCheckpointSource,
    RelationalDurableRecoverySource, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::cold_recovery::capture_external_checkpoint;
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, naming_page_reads, observe_branch,
    publish_group_partition, reopen_named, GROUP_SCHEME,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_ack_isolation_and_rebind::{capture_only, try_ack};
use super::naming_branch_root_and_dissolve::fork_retained_at;

fn drop_one_group_log_value_chunk_page(
    ck: &RelationalNativeCheckpoint,
) -> RelationalNativeCheckpoint {
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let pages = rows(field(naming, "group_log"));
    let mut kept: Vec<&[u8]> = Vec::new();
    let mut dropped_chunk = false;
    let mut kept_primary = 0usize;
    for page in pages {
        let first: Vec<u8> = rmp_serde::from_slice(field(page, "first")).unwrap();
        if first[0] == 15 && !dropped_chunk {
            dropped_chunk = true;
            continue;
        }
        if first[0] == 14 {
            kept_primary += 1;
        }
        kept.push(page);
    }
    assert!(kept_primary > 0, "expected primary GROUP_LOG page");
    assert!(dropped_chunk, "fixture must emit VALUE_CHUNK pages");
    let new_naming = replace_field(naming, "group_log", &array(&kept));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + new_body.len());
    bytes.extend_from_slice(CHECKPOINT_MAGIC);
    bytes.extend_from_slice(&new_body);
    RelationalNativeCheckpoint::from_untrusted_bytes(bytes)
}

#[test]
fn long_branch_id_group_transition_checkpoints_reopens_and_replays_history() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, _, _) = create_named_entities(request, &producer, &main, &["r7-seed".into()]);
    let long_name = format!("r7-long-branch-{}", "x".repeat(185));
    assert_eq!(long_name.len(), 200);
    let long_branch = BranchId(long_name);
    let main_obs = observe_branch(&producer, &main);
    fork_retained_at(request, &producer, &main_obs, &long_branch).unwrap();

    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &long_branch,
        &["r7-m1".into(), "r7-m2".into()],
    );
    let _ = publish_group_partition(
        request,
        &producer,
        &long_branch,
        entities[0],
        "r7-long-group",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Interior),
        ]],
    );
    let basis_c1 = observe_branch(&producer, &long_branch);
    let g1 = producer
        .with_persistent_names_at_observation(request, &basis_c1.observation(), |v| {
            v.group_of_member(GROUP_SCHEME, &names[0]).unwrap()
        })
        .unwrap();

    // Capture C1 + ACK, then publish another group transition and capture C2
    // to verify bounded digest keys work across both fresh capture and C1->C2 carry.
    let ck1 = capture_only(request, &producer, &medium, &[]);
    medium.file_checkpoint(ck1.clone());
    try_ack(request, &ck1).unwrap();

    let _ = publish_group_partition(
        request,
        &producer,
        &long_branch,
        entities[0],
        "r7-long-group-2",
        vec![vec![
            (names[1], AggregateMemberRole::Anchor),
            (names[0], AggregateMemberRole::Interior),
        ]],
    );

    // Historical fork at `basis_c1` on the cold-backed 200-byte branch replays
    // the cold GROUP_LOG entry keyed by the bounded branch digest cleanly.
    let child = BranchId("r7-hist-child".into());
    fork_retained_at(request, &producer, &basis_c1, &child).unwrap();
    let child_obs = observe_branch(&producer, &child);
    producer
        .with_persistent_names_at_observation(request, &child_obs.observation(), |v| {
            assert_eq!(v.group_of_member(GROUP_SCHEME, &names[0]), Some(g1));
        })
        .unwrap();

    let (ck2, source) = capture_external_checkpoint(request, &producer);
    let reopened = reopen_named(request, ck2, source.clone(), &medium, u64::MAX / 4);
    assert_eq!(
        naming_page_reads(&source),
        0,
        "reopen must read zero naming pages"
    );
    let reopened_child = observe_branch(&reopened, &child);
    reopened
        .with_persistent_names_at_observation(request, &reopened_child.observation(), |v| {
            assert_eq!(v.group_of_member(GROUP_SCHEME, &names[0]), Some(g1));
        })
        .unwrap();
}

fn assert_corrupt_read_free(
    request: &ExecutionRequest<'_, '_>,
    medium: &std::sync::Arc<TestDurableMedium>,
    ck: &RelationalNativeCheckpoint,
    tampered: RelationalNativeCheckpoint,
    source: &std::sync::Arc<
        worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource,
    >,
) {
    let mut target = durable_named_runtime(medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            tampered,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
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
            .expect_err("tampered CHK24 continuation must refuse CorruptCheckpoint"),
    };
    assert!(
        matches!(
            err,
            RelationalRecoveryStop::Durability(ref d)
                if d.class == RecoveryFailureClass::CorruptCheckpoint
        ),
        "expected CorruptCheckpoint, got {err:?}"
    );
    assert_eq!(naming_page_reads(source), 0, "must refuse read-free");
}

#[test]
fn tampered_chk24_dropping_group_log_value_chunk_page_refuses_before_install() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let (ck, source, _, _, _) = capture_mixed_inline_and_chunked_group_log(request, &medium);
    let tampered = drop_one_group_log_value_chunk_page(&ck);
    assert_corrupt_read_free(request, &medium, &ck, tampered, &source);
}

fn forged_continuation_digest(key: &[u8], count: u32) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"worth.naming.page.continuations.v1\0");
    hash.update(&(key.len() as u32).to_be_bytes());
    hash.update(key);
    hash.update(&count.to_be_bytes());
    hash.finalize().into()
}

fn reassign_chunk_descriptors_to_inline_row(
    ck: &RelationalNativeCheckpoint,
    forge_digest: bool,
) -> RelationalNativeCheckpoint {
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let pages = rows(field(naming, "group_log"));
    let mut inline_base: Option<Vec<u8>> = None;
    let mut edited: Vec<Vec<u8>> = Vec::with_capacity(pages.len());
    let mut total_chunks = 0u32;
    for page in pages {
        let first: Vec<u8> = rmp_serde::from_slice(field(page, "first")).unwrap();
        if first[0] == 14 {
            assert!(
                inline_base.is_none(),
                "fixture expects one primary GROUP_LOG page carrying inline + chunked rows"
            );
            inline_base = Some(first);
            edited.push(page.to_vec());
            continue;
        }
        let target_base = inline_base.as_ref().expect("primary page precedes chunks");
        let last: Vec<u8> = rmp_serde::from_slice(field(page, "last")).unwrap();
        let entries: u32 = rmp_serde::from_slice(field(page, "entries")).unwrap();
        total_chunks += entries;
        let rewrite = |orig: &[u8]| -> Vec<u8> {
            let mut out = Vec::with_capacity(1 + 2 + target_base.len() + 4);
            out.push(15u8);
            out.extend_from_slice(&(target_base.len() as u16).to_be_bytes());
            out.extend_from_slice(target_base);
            out.extend_from_slice(&orig[orig.len() - 4..]);
            out
        };
        let p = replace_field(page, "first", &rmp_serde::to_vec(&rewrite(&first)).unwrap());
        let p = replace_field(&p, "last", &rmp_serde::to_vec(&rewrite(&last)).unwrap());
        edited.push(p);
    }
    assert!(total_chunks > 0, "fixture must emit VALUE_CHUNK pages");
    if forge_digest {
        let digest = forged_continuation_digest(inline_base.as_ref().unwrap(), total_chunks);
        let encoded = rmp_serde::to_vec(&Some(digest)).unwrap();
        edited[0] = replace_field(&edited[0], "continuation_digest", &encoded);
    }
    let refs: Vec<&[u8]> = edited.iter().map(Vec::as_slice).collect();
    let new_naming = replace_field(naming, "group_log", &array(&refs));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + new_body.len());
    bytes.extend_from_slice(CHECKPOINT_MAGIC);
    bytes.extend_from_slice(&new_body);
    RelationalNativeCheckpoint::from_untrusted_bytes(bytes)
}

fn capture_mixed_inline_and_chunked_group_log(
    request: &ExecutionRequest<'_, '_>,
    medium: &std::sync::Arc<TestDurableMedium>,
) -> (
    RelationalNativeCheckpoint,
    std::sync::Arc<
        worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource,
    >,
    BranchId,
    worth_relational::facade::identity::EntityId,
    Vec<worth_relational::facade::lineage::PersistentNameIdentity>,
){
    let producer = durable_named_runtime(medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();
    let labels: Vec<String> = (0..900)
        .map(|i| format!("r8-mixed-member-{i:04}"))
        .collect();
    let (_, entities, names) = create_named_entities(request, &producer, &main, &labels);
    publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "r8-inline-row",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Interior),
        ]],
    );
    let big_members: Vec<_> = names[2..]
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let role = if i == 0 {
                AggregateMemberRole::Anchor
            } else {
                AggregateMemberRole::Interior
            };
            (name, role)
        })
        .collect();
    publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "r8-chunked-row",
        vec![big_members],
    );
    let (ck, source) = capture_external_checkpoint(request, &producer);
    (ck, source, main, entities[0], names)
}

#[test]
fn tampered_chk24_reassigning_group_log_chunks_to_inline_row_refuses_before_install() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let (ck, source, _, _, _) = capture_mixed_inline_and_chunked_group_log(request, &medium);
    let tampered = reassign_chunk_descriptors_to_inline_row(&ck, false);
    assert_corrupt_read_free(request, &medium, &ck, tampered, &source);
}

#[test]
fn tampered_chk24_forging_both_continuation_digest_and_chunk_roster_fails_closed_on_fault() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let (ck, source, main, entity0, names) =
        capture_mixed_inline_and_chunked_group_log(request, &medium);
    let forged = reassign_chunk_descriptors_to_inline_row(&ck, true);
    // Stage 1 (G7 read-free reopen): consistent forged manifest installs with 0 naming page reads.
    let mut reopened = durable_named_runtime(&medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            ck.covered_position().unwrap(),
            ck.covered_branch_transitions().unwrap(),
            forged,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("forged consistent manifest plans read-free");
    reopened
        .durability_recovery()
        .recover(request, plan)
        .expect("G7 Stage 1 reopen must succeed when forged manifest digest and roster agree");
    assert_eq!(
        naming_page_reads(&source),
        0,
        "G7 Stage 1 reopen must remain read-free even when manifest digest and roster agree"
    );
    let retained = observe_branch(&reopened, &main);
    publish_group_partition(
        request,
        &reopened,
        &main,
        entity0,
        "r8-post-reopen-advance",
        vec![vec![
            (names[1], AggregateMemberRole::Anchor),
            (names[0], AggregateMemberRole::Interior),
        ]],
    );
    // Stage 2 (first fault of forged primary GROUP_LOG page): admit() detects digest mismatch.
    let reads_before_fault = naming_page_reads(&source);
    let child = BranchId("r8-forged-child".into());
    let err = fork_retained_at(request, &reopened, &retained, &child)
        .expect_err("forged continuation_digest must fail closed on first GROUP_LOG fault");
    assert!(
        err.contains("RetentionOwnerUnavailable"),
        "expected fail-closed preflight refusal on first fault, got: {err}"
    );
    assert!(
        naming_page_reads(&source) > reads_before_fault,
        "Stage 2 must fault and reject the forged primary GROUP_LOG page via admit()"
    );
    assert!(
        reopened.branch_identity(&child).is_err(),
        "failed Stage 2 fault must leave no forked branch or corrupt group state"
    );
}

//! Round 1 hardening courts for Store M4.3f (`[P0-1]`, `[P1-2]`, `[P1-3]`).
use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalDurableCheckpointSource,
    RelationalDurableRecoverySource, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, PersistentNameKey, PersistentNameResolution, PersistentNameSelection,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_relational::facade::transactions::{
    CommitPreparationReason, EntityMutationIntent, MutationIntent, TransactionCommitError,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, group_scheme, observe_branch,
    publish_group_partition, reopen_named, GROUP_SCHEME,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, CHECKPOINT_MAGIC};

fn tamper_manifest_naming(
    checkpoint: &RelationalNativeCheckpoint,
    mutate_naming: impl FnOnce(&[u8]) -> Vec<u8>,
) -> RelationalNativeCheckpoint {
    let body = checkpoint
        .bytes()
        .strip_prefix(CHECKPOINT_MAGIC)
        .expect("CHK24 magic");
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let new_naming = mutate_naming(naming);
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + new_body.len());
    bytes.extend_from_slice(CHECKPOINT_MAGIC);
    bytes.extend_from_slice(&new_body);
    RelationalNativeCheckpoint::from_untrusted_bytes(bytes)
}

#[test]
fn tampered_chk24_dropping_branch_naming_root_refuses_corrupt_checkpoint() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();
    let (_, entities, names) = create_named_entities(
        request,
        &producer,
        &main,
        &["tamp-1".into(), "tamp-2".into()],
    );
    publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "tamp-grp",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Interior),
        ]],
    );
    let (c1, source) = capture_external_checkpoint(request, &producer);

    // Strip all branches from `manifest.naming.branches` (attempting silent erasure).
    let tampered = tamper_manifest_naming(&c1, |naming| {
        let empty = array(&[]);
        replace_field(naming, "branches", &empty)
    });
    let mut target = durable_named_runtime(&medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            c1.covered_position().unwrap(),
            c1.covered_branch_transitions().unwrap(),
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
            .expect_err("tampered CHK24 dropping branch naming root must refuse"),
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

#[test]
fn isolated_member_groups_damage_refuses_before_append_and_live_names_is_fallible() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let mut all_entities = Vec::new();
    let mut all_names = Vec::new();
    for b in 0..35usize {
        let keys: Vec<String> = (0..10).map(|i| format!("iso-{b:02}-{i}")).collect();
        let (_, e, n) = create_named_entities(request, &producer, &main, &keys);
        all_entities.extend(e);
        all_names.extend(n);
    }
    let members: Vec<_> = all_names
        .iter()
        .enumerate()
        .map(|(i, &n)| {
            let role = if i == 0 {
                AggregateMemberRole::Anchor
            } else {
                AggregateMemberRole::Interior
            };
            (n, role)
        })
        .collect();
    publish_group_partition(
        request,
        &producer,
        &main,
        all_entities[0],
        "iso-g",
        vec![members],
    );

    let (c1, _) = capture_external_checkpoint(request, &producer);
    let source = immutable_source(&c1, &medium);
    // Find a naming page whose keys are strictly in `INDEX_MEMBER_GROUPS` (tag 10).
    let dir = super::selected_directory::read_native(&c1);
    let member_groups_page = dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .find(|p| p.first.first() == Some(&10) && p.last.first() == Some(&10))
        .map(|p| p.member)
        .expect("dedicated INDEX_MEMBER_GROUPS page");

    let reopened = reopen_named(request, c1.clone(), source.clone(), &medium, u64::MAX / 4);
    let head_before = observe_branch(&reopened, &main).observation().commit_id();
    source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(member_groups_page), Cause::Damaged);

    // Group update touching member_groups must fail in Preparation BEFORE durable append.
    let basis = observe_branch(&reopened, &main);
    let mut tx = reopened
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("iso-dmg").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: all_entities[0],
                fields: field_patch("name", "name", "iso-upd"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(
        group_scheme(),
        Vec::new(),
        vec![vec![(all_names[0], AggregateMemberRole::Anchor)]],
    );
    tx.attach_naming_plan(plan);
    let err = tx.commit(request, &reopened).unwrap_err();
    assert!(
        matches!(
            err,
            TransactionCommitError::Preparation { ref error, .. }
                if matches!(error.reason(), CommitPreparationReason::StorageUnavailable(_))
        ),
        "isolated member_groups damage must refuse in preparation, got {err:?}"
    );
    assert_eq!(
        observe_branch(&reopened, &main).observation().commit_id(),
        head_before,
        "head commit must NOT advance on damaged member_groups page"
    );

    // Clearing the fault proves the branch name index was NOT poisoned.
    source.failures.lock().unwrap().clear();
    let after = observe_branch(&reopened, &main);
    reopened
        .with_persistent_names_at_observation(request, &after.observation(), |view| {
            assert!(view.group_of_member(GROUP_SCHEME, &all_names[0]).is_some());
        })
        .unwrap();

    // Reopen fresh with a damaged LIVE_HOLDER (tag 3) page so it is not cached:
    // `branch_live_persistent_names` must return Err(Storage(Damaged)) instead of panicking.
    let live_holder_page = dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .find(|p| p.first.first() == Some(&3) || p.last.first() == Some(&3))
        .map(|p| p.member)
        .expect("LIVE_HOLDER page");
    let source2 = immutable_source(&c1, &medium);
    let reopened2 = reopen_named(request, c1, source2.clone(), &medium, u64::MAX / 4);
    source2
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(live_holder_page), Cause::Damaged);
    let live_err = reopened2
        .branch_live_persistent_names(&main)
        .expect_err("damaged LIVE_HOLDER page must return Err instead of panicking");
    assert_eq!(format!("{live_err:?}"), "Storage(Damaged)");
}

#[test]
fn cold_reopen_retained_historical_fork_preserves_names_and_groups_at_cutoff() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, e1, n1) =
        create_named_entities(request, &producer, &main, &["hf-1".into(), "hf-2".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e1[0],
        "hf-g1",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Interior),
        ]],
    );
    let (c1, source) = capture_external_checkpoint(request, &producer);
    let reopened = reopen_named(request, c1, source, &medium, u64::MAX / 4);

    // Hold a retained basis on `main` at C2 (post-reopen), then advance `main` to C3 & C4.
    let services = reopened.owner_component_services();
    let retained_c2 = services
        .basis_port()
        .admit_branch_basis(&reopened.main_branch_identity())
        .unwrap();
    delete_entity(request, &reopened, &main, e1[1]);
    let (c4, e3, n3) = create_named_entities(request, &reopened, &main, &["hf-3".into()]);
    publish_group_partition(
        request,
        &reopened,
        &main,
        e3[0],
        "hf-g2",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n3[0], AggregateMemberRole::Interior),
        ]],
    );

    // Fork retained historical branch from C2 (strictly behind `main`'s head!).
    let hist_fork = BranchId("hist-fork".into());
    let fork_src = services
        .fork_port()
        .admit_retained_fork_source(&retained_c2)
        .unwrap();
    services
        .fork_port()
        .fork_retained_branch(request, hist_fork.clone(), fork_src)
        .expect("cold-reopened retained historical fork must succeed");

    // Verify `hist-fork` sees exact C2 names and groups (hf-2 live, hf-3 absent, G1={hf-1,hf-2}).
    let fork_obs = observe_branch(&reopened, &hist_fork);
    reopened
        .with_persistent_names_at_observation(request, &fork_obs.observation(), |view| {
            assert!(matches!(
                view.resolve(
                    PersistentNameKey::Entity(n1[1]),
                    &PersistentNameSelection::Exact,
                )
                .result,
                Ok(PersistentNameResolution::Exact(_))
            ));
            assert!(view
                .resolve(
                    PersistentNameKey::Entity(n3[0]),
                    &PersistentNameSelection::Exact,
                )
                .result
                .is_err());
            let g = view
                .group_of_member(GROUP_SCHEME, &n1[1])
                .expect("hf-2 still in G1 at C2");
            let members = view.group_members(g).unwrap();
            assert_eq!(members.len(), 2);
            assert!(members.contains(&n1[0]) && members.contains(&n1[1]));
        })
        .unwrap();

    // New named + group commits on `hist-fork` succeed cleanly.
    let (_, ef, nf) = create_named_entities(request, &reopened, &hist_fork, &["hf-child".into()]);
    publish_group_partition(
        request,
        &reopened,
        &hist_fork,
        ef[0],
        "hf-fork-g",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Interior),
            (nf[0], AggregateMemberRole::Interior),
        ]],
    );

    // Advancing main's retention floor past C2 causes another retained fork at C2 to refuse cleanly.
    reopened
        .advance_naming_retention_floor(&main, c4.commit.commit_id)
        .unwrap();
    let fork_src2 = services
        .fork_port()
        .admit_retained_fork_source(&retained_c2)
        .unwrap();
    let denied = services.fork_port().fork_retained_branch(
        request,
        BranchId("hist-fork-pruned".into()),
        fork_src2,
    );
    assert!(
        denied.is_err(),
        "historical fork below retention floor must refuse before installing branch"
    );
    assert!(reopened
        .branch_identity(&BranchId("hist-fork-pruned".into()))
        .is_err());
}

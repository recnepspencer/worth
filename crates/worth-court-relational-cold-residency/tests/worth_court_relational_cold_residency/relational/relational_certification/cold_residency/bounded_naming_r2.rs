//! Round 2 hardening courts for Store M4.3f (`[P0-1]` and `[P1-2]`).
use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalDurableCheckpointSource,
    RelationalDurableRecoverySource, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{CommitNamingPlan, PersistentNameKey};
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
    create_named_entities, durable_named_runtime, group_scheme, observe_branch,
    publish_group_partition, reopen_named, GROUP_SCHEME,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};

pub(super) fn fork_retained_at(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    target: &BranchId,
) -> Result<(), String> {
    let services = runtime.owner_component_services();
    let src = services
        .fork_port()
        .admit_retained_fork_source(basis)
        .map_err(|e| format!("{e:?}"))?;
    services
        .fork_port()
        .fork_retained_branch(request, target.clone(), src)
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn tampered_chk24_emptying_single_branch_root_with_other_branch_intact_refuses_corrupt_checkpoint()
{
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (_, e_main, n_main) =
        create_named_entities(request, &producer, &main, &["a-1".into(), "a-2".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e_main[0],
        "a-grp",
        vec![vec![
            (n_main[0], AggregateMemberRole::Anchor),
            (n_main[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let branch_b = BranchId("branch-b".into());
    let main_basis = observe_branch(&producer, &main);
    fork_retained_at(request, &producer, &main_basis, &branch_b).unwrap();
    let (_, e_b, n_b) =
        create_named_entities(request, &producer, &branch_b, &["b-1".into(), "b-2".into()]);
    publish_group_partition(
        request,
        &producer,
        &branch_b,
        e_b[0],
        "b-grp",
        vec![vec![
            (n_b[0], AggregateMemberRole::Anchor),
            (n_b[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let (c1, source) = capture_external_checkpoint(request, &producer);

    // Replace ONLY `branch-b`'s BranchNamingRoot with an empty root while leaving `main` intact.
    let body = c1.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let branch_rows = rows(field(naming, "branches"));
    let nil = rmp_serde::to_vec(&Option::<u8>::None).unwrap();
    let empty_pages = array(&[]);
    let mut edited_storage: Vec<Vec<u8>> = Vec::with_capacity(branch_rows.len());
    let mut emptied_b = false;
    for row in branch_rows {
        let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if name == branch_b {
            let r = replace_field(row, "naming", &nil);
            let r = replace_field(&r, "index", &nil);
            let r = replace_field(&r, "groups", &nil);
            let r = replace_field(&r, "pages", &empty_pages);
            edited_storage.push(r);
            emptied_b = true;
        } else {
            edited_storage.push(row.to_vec());
        }
    }
    assert!(
        emptied_b,
        "branch-b root must exist in manifest.naming.branches"
    );
    let refs: Vec<&[u8]> = edited_storage.iter().map(Vec::as_slice).collect();
    let new_naming = replace_field(naming, "branches", &array(&refs));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    let tampered =
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat());

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
            .expect_err("emptying branch-b naming root while main stays intact must refuse"),
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
fn dissolve_only_commit_with_damaged_member_owner_refuses_before_append() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let mut all_entities = Vec::new();
    let mut all_names = Vec::new();
    for b in 0..35usize {
        let keys: Vec<String> = (0..10).map(|i| format!("dslv-{b:02}-{i}")).collect();
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
                AggregateMemberRole::Ordinary
            };
            (n, role)
        })
        .collect();
    publish_group_partition(
        request,
        &producer,
        &main,
        all_entities[0],
        "dslv-g",
        vec![members],
    );
    let group_id = match producer
        .with_persistent_names_at_observation(
            request,
            &observe_branch(&producer, &main).observation(),
            |v| v.group_of_member(GROUP_SCHEME, &all_names[0]).unwrap(),
        )
        .unwrap()
    {
        PersistentNameKey::Group(id) => id,
        other => panic!("expected group key, got {other:?}"),
    };

    let (c1, _) = capture_external_checkpoint(request, &producer);
    let dir = super::selected_directory::read_native(&c1);
    let owner_page = dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .find(|p| p.first.first() == Some(&13) && p.last.first() == Some(&13))
        .map(|p| p.member)
        .expect("dedicated GROUP_MEMBER_OWNER (tag 13) page");
    let source = immutable_source(&c1, &medium);
    let reopened = reopen_named(request, c1, source.clone(), &medium, u64::MAX / 4);
    let head_before = observe_branch(&reopened, &main).observation().commit_id();
    source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(owner_page), Cause::Damaged);

    // Dissolve-only commit: empty next-state assignments, keeping all members live.
    let basis = observe_branch(&reopened, &main);
    let mut tx = reopened
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("dslv-only").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: all_entities[0],
                fields: field_patch("name", "name", "dslv-upd"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(group_scheme(), vec![group_id], Vec::new());
    tx.attach_naming_plan(plan);
    let err = tx.commit(request, &reopened).unwrap_err();
    assert!(
        matches!(
            err,
            TransactionCommitError::Preparation { ref error, .. }
                if matches!(error.reason(), CommitPreparationReason::StorageUnavailable(_))
        ),
        "dissolve-only with damaged GROUP_MEMBER_OWNER must refuse in preparation, got {err:?}"
    );
    assert_eq!(
        observe_branch(&reopened, &main).observation().commit_id(),
        head_before,
        "head must not advance on damaged GROUP_MEMBER_OWNER"
    );

    // Clearing the fault proves group state was NOT poisoned into Unavailable.
    source.failures.lock().unwrap().clear();
    let basis2 = observe_branch(&reopened, &main);
    let mut tx2 = reopened
        .begin_branch_transaction(&basis2, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx2.push_batch(
        WorkerIntentBatch::new("dslv-ok").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: all_entities[0],
                fields: field_patch("name", "name", "dslv-ok"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan2 = CommitNamingPlan::default();
    plan2.declare_group_partition(group_scheme(), vec![group_id], Vec::new());
    tx2.attach_naming_plan(plan2);
    tx2.commit(request, &reopened)
        .expect("dissolve succeeds once damaged page heals");
}

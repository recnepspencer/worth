//! Round 3 hardening court for Store M4.3f (`[P1-3]`):
//! - First group write in a previously unused scheme installs read-free
//!   (`base = None`) and commits cleanly even when another scheme's cold group
//!   page is damaged, while touching the damaged scheme still refuses before
//!   durable append.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
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
    publish_group_partition, reopen_named,
};

#[test]
fn unused_group_scheme_commits_cleanly_while_existing_scheme_cold_page_is_damaged() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let mut all_entities = Vec::new();
    let mut all_names = Vec::new();
    for b in 0..35usize {
        let keys: Vec<String> = (0..10).map(|i| format!("s1-{b:02}-{i}")).collect();
        let (_, e, n) = create_named_entities(request, &producer, &main, &keys);
        all_entities.extend(e);
        all_names.extend(n);
    }
    let s1_members: Vec<_> = all_names
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
        "s1-big",
        vec![s1_members],
    );

    let (c1, _) = capture_external_checkpoint(request, &producer);
    let dir = super::selected_directory::read_native(&c1);
    let s1_owner_page = dir
        .naming
        .branches
        .iter()
        .flat_map(|b| &b.pages)
        .find(|p| p.first.first() == Some(&13) && p.last.first() == Some(&13))
        .map(|p| p.member)
        .expect("dedicated S1 GROUP_MEMBER_OWNER page");
    let source = immutable_source(&c1, &medium);
    let reopened = reopen_named(request, c1, source.clone(), &medium, u64::MAX / 4);

    // Damage S1's cold group page, then commit the FIRST group write in unused scheme S2.
    source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Member(s1_owner_page), Cause::Damaged);
    let s2_id = "worth.m43f.group.s2.v1";
    let s2_scheme = AggregateGroupScheme::new(
        AggregateGroupSchemeId::new(s2_id).unwrap(),
        AggregateContinuityRule::Anchor,
    );
    let basis = observe_branch(&reopened, &main);
    let mut tx = reopened
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("s2-first").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: all_entities[0],
                fields: field_patch("name", "name", "s2-first"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(
        s2_scheme,
        Vec::new(),
        vec![vec![
            (all_names[0], AggregateMemberRole::Anchor),
            (all_names[1], AggregateMemberRole::Ordinary),
        ]],
    );
    tx.attach_naming_plan(plan);
    tx.commit(request, &reopened).expect(
        "first write to unused scheme S2 must install read-free without touching damaged S1",
    );

    let after_s2 = observe_branch(&reopened, &main);
    reopened
        .with_persistent_names_at_observation(request, &after_s2.observation(), |v| {
            assert!(matches!(
                v.group_of_member(s2_id, &all_names[0]),
                Some(PersistentNameKey::Group(_))
            ));
        })
        .unwrap();

    // Touching damaged S1 itself still refuses cleanly before durable append.
    let mut tx_s1 = reopened
        .begin_branch_transaction(&after_s2, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx_s1
        .push_batch(
            WorkerIntentBatch::new("s1-touch").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: all_entities[0],
                    fields: field_patch("name", "name", "s1-touch"),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let mut plan_s1 = CommitNamingPlan::default();
    plan_s1.declare_group_partition(
        group_scheme(),
        Vec::new(),
        vec![vec![(all_names[0], AggregateMemberRole::Anchor)]],
    );
    tx_s1.attach_naming_plan(plan_s1);
    let err = tx_s1.commit(request, &reopened).unwrap_err();
    assert!(matches!(
        err,
        TransactionCommitError::Preparation { ref error, .. }
            if matches!(error.reason(), CommitPreparationReason::StorageUnavailable(_))
    ));
}

//! Regression court proving that retaining a forked Relational branch
//! preserves ALL inherited parent versions (creation + updates on `main`)
//! in chronological order instead of fabricating a synthetic single `Created`
//! entry for untouched inherited records.

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, VersionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_runtime_world::facade::{
    CanonicalKeyDomain, HistoricalBasisRetentionKind, HistoricalQueryContract,
    ORDERING_CANONICAL_ASC, PROJECTION_CANONICAL_FULL,
};
use worth_store_application::{
    encode_canonical_entity_payload, encode_canonical_history_version, HistoricalChunkGeometry,
};

use super::{
    canonical_logical_key, close_durable_home, commit_and_release, drain_cursor, open_durable_home,
    test_budget,
};

#[test]
fn s5_forked_branch_retains_all_inherited_parent_versions_for_untouched_records() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let main_branch = BranchId("main".into());
    let feature_branch = BranchId("feature-inherited".into());
    let schema_id = SchemaId("m4.relational.fork.history".into());
    let schema_ver = SchemaVersionId(1);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 2,
        inline_history_limit: 1,
        continuation_page_versions: 1,
        layered_initial: true,
    };

    let home = open_durable_home(dir.path(), &request);
    let runtime = home.relational().runtime();

    // 1. Create an entity on `main`, then commit TWO updates to it on `main` (3 real versions!).
    commit_and_release(runtime, &execution, &main_branch, crate::batches::create(0));
    let main_id = runtime.branch_identity(&main_branch).unwrap();
    let (_, init_obs) = runtime.observe_branch(&main_id).unwrap();
    let init_view = runtime
        .read_truth()
        .read_observation(&execution, &init_obs.observation())
        .unwrap()
        .unwrap();
    let inherited_entity: EntityId = init_view
        .entities()
        .iter()
        .find(|e| e.lifecycle == RecordLifecycleState::Live && e.kind.kind_id == KindId(1))
        .expect("created entity on main")
        .entity_id;
    drop(init_obs);

    commit_and_release(
        runtime,
        &execution,
        &main_branch,
        crate::batches::update(inherited_entity, 201),
    );
    commit_and_release(
        runtime,
        &execution,
        &main_branch,
        crate::batches::update(inherited_entity, 202),
    );

    // 2. Capture independent `main` ground-truth history for `inherited_entity` BEFORE fork.
    let (_main_basis, main_obs) = runtime.observe_branch(&main_id).unwrap();
    let fork_ver = VersionId(main_obs.descriptor().truth_version().as_u64());
    let main_entries = runtime
        .history()
        .entity_aspect_history(&execution, &main_branch, inherited_entity, None)
        .unwrap();
    assert_eq!(
        main_entries.len(),
        3,
        "main must record Created + 2 Updated versions"
    );
    let expected_history: Vec<Vec<u8>> = main_entries
        .into_iter()
        .filter(|e| e.origin.version_id.0 <= fork_ver.0)
        .map(|e| {
            let hr = runtime
                .read_truth()
                .visible_entity_at_version(&execution, inherited_entity, e.origin.version_id)
                .unwrap();
            let vp = encode_canonical_entity_payload(
                hr.as_ref()
                    .and_then(|r| r.authoritative_aspect_state.as_ref()),
            );
            encode_canonical_history_version(
                e.origin.commit_id,
                e.origin.version_id,
                e.origin.structural_change,
                &e.origin.changed_aspects,
                &vp,
            )
        })
        .collect();
    assert_eq!(expected_history.len(), 3);
    assert!(expected_history[1].windows(11).any(|w| w == b"renamed-201"));
    assert!(expected_history[2].windows(11).any(|w| w == b"renamed-202"));
    drop(main_obs);

    // 3. Fork `feature-inherited` from `main` and commit an UNRELATED entity on `feature`
    //    (leaving `inherited_entity` completely untouched on `feature`). Also advance `main`
    //    with a post-fork update (999) that MUST NOT appear on `feature`!
    let (_, fork_basis) = runtime
        .observe_fork_source(&main_branch)
        .expect("observe fork source on main");
    runtime
        .fork_branch(&execution, feature_branch.clone(), fork_basis)
        .expect("fork feature branch from main");
    commit_and_release(
        runtime,
        &execution,
        &feature_branch,
        crate::batches::create(1),
    );
    commit_and_release(
        runtime,
        &execution,
        &main_branch,
        crate::batches::update(inherited_entity, 999),
    );

    // 4. Retain `feature-inherited` and stream `Full` historical cursor!
    let store = home.historical_basis_store(budget).unwrap();
    let receipt = store
        .retain_relational_branch_revision(
            runtime,
            &execution,
            &feature_branch,
            schema_id,
            schema_ver,
            &[KindId(1)],
            &[KindId(2)],
            HistoricalBasisRetentionKind::PinLease,
            geom,
        )
        .expect("retain forked feature branch revision");

    let query = HistoricalQueryContract::new(
        receipt.schema_version.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(1)),
        PROJECTION_CANONICAL_FULL,
        ORDERING_CANONICAL_ASC,
    );
    let mut cursor = store
        .open_historical_cursor(
            &execution,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            query,
            &receipt.schema_version,
        )
        .expect("open Full historical cursor on retained fork");
    let rows = drain_cursor(&mut cursor, &execution, 2);
    drop(cursor);

    assert_eq!(
        rows.len(),
        2,
        "fork holds inherited entity + new feature entity"
    );
    let mut expected_key = Vec::with_capacity(20);
    expected_key.extend_from_slice(&KindId(1).0.to_be_bytes());
    expected_key.extend_from_slice(&inherited_entity.partition_id.0.to_be_bytes());
    expected_key.extend_from_slice(&inherited_entity.local_slot.0.to_be_bytes());
    expected_key.extend_from_slice(&inherited_entity.generation.0.to_be_bytes());
    let inherited_row = rows
        .iter()
        .find(|row| row.logical_key == expected_key)
        .expect("inherited entity present in retained fork");
    assert_eq!(
        inherited_row.history_versions, expected_history,
        "retained fork MUST preserve all 3 real inherited main versions (never fabricate a single Created entry)"
    );
    assert!(
        inherited_row
            .history_versions
            .iter()
            .all(|h| !h.windows(11).any(|w| w == b"renamed-999")),
        "post-fork update on main MUST NOT leak into retained fork history"
    );

    drop(store);
    close_durable_home(home, &request);
}

#[test]
fn s5_non_main_parent_branch_ancestry_replays_exact_parent_versions_for_child_fork() {
    use crate::batches::{
        field_patch, ClientKey, CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent,
        PartitionId, UpdateEntityFieldsIntent, WorkerIntentBatch,
    };

    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let main_branch = BranchId("main".into());
    let parent_branch = BranchId("parent_branch".into());
    let child_branch = BranchId("child_branch".into());
    let home = open_durable_home(dir.path(), &request);
    let runtime = home.relational().runtime();

    // 1. Seed `main` so it has a forkable base commit, then fork `parent_branch` from `main`,
    //    and create `"parent-v1"` + update to `"parent-v2"` exclusively on `parent_branch`.
    commit_and_release(runtime, &execution, &main_branch, crate::batches::create(0));
    let main_id = runtime.branch_identity(&main_branch).unwrap();
    let (_, m_obs) = runtime.observe_branch(&main_id).unwrap();
    let main_entity = runtime
        .read_truth()
        .read_observation(&execution, &m_obs.observation())
        .unwrap()
        .unwrap()
        .entities()
        .iter()
        .find(|e| e.lifecycle == RecordLifecycleState::Live && e.kind.kind_id == KindId(1))
        .expect("main seed entity")
        .entity_id;
    drop(m_obs);

    let (_, main_fork) = runtime.observe_fork_source(&main_branch).unwrap();
    runtime
        .fork_branch(&execution, parent_branch.clone(), main_fork)
        .unwrap();
    let create_v1 = WorkerIntentBatch::new("create-parent-v1").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("parent-v1"),
            fields: field_patch("name", "parent-v1"),
        }),
    ));
    commit_and_release(runtime, &execution, &parent_branch, create_v1);
    let parent_id = runtime.branch_identity(&parent_branch).unwrap();
    let (_, p_obs) = runtime.observe_branch(&parent_id).unwrap();
    let p_view = runtime
        .read_truth()
        .read_observation(&execution, &p_obs.observation())
        .unwrap()
        .unwrap();
    let parent_entity = p_view
        .entities()
        .iter()
        .find(|e| {
            e.lifecycle == RecordLifecycleState::Live
                && e.kind.kind_id == KindId(1)
                && e.entity_id != main_entity
        })
        .expect("parent_branch entity")
        .entity_id;
    drop(p_obs);
    let update_v2 = WorkerIntentBatch::new("update-parent-v2").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: parent_entity,
            fields: field_patch("name", "parent-v2"),
        }),
    ));
    commit_and_release(runtime, &execution, &parent_branch, update_v2);

    // 2. Fork `child_branch` from `parent_branch`, commit an unrelated entity on `child_branch`,
    //    and retain `child_branch`.
    let (_, parent_fork) = runtime.observe_fork_source(&parent_branch).unwrap();
    runtime
        .fork_branch(&execution, child_branch.clone(), parent_fork)
        .unwrap();
    commit_and_release(
        runtime,
        &execution,
        &child_branch,
        crate::batches::create(77),
    );

    let store = home.historical_basis_store(budget).unwrap();
    let receipt = store
        .retain_relational_branch_revision(
            runtime,
            &execution,
            &child_branch,
            SchemaId("m4.nonmain.ancestry".into()),
            SchemaVersionId(1),
            &[KindId(1)],
            &[KindId(2)],
            HistoricalBasisRetentionKind::PinLease,
            HistoricalChunkGeometry {
                rows_per_chunk: 4,
                inline_history_limit: 4,
                continuation_page_versions: 2,
                layered_initial: true,
            },
        )
        .expect("retain child_branch forked from parent_branch");
    let query = HistoricalQueryContract::new(
        receipt.schema_version.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(1)),
        PROJECTION_CANONICAL_FULL,
        ORDERING_CANONICAL_ASC,
    );
    let mut cursor = store
        .open_historical_cursor(
            &execution,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            query,
            &receipt.schema_version,
        )
        .unwrap();
    let rows = drain_cursor(&mut cursor, &execution, 4);
    drop(cursor);

    let key = canonical_logical_key(
        1,
        parent_entity.partition_id.0,
        parent_entity.local_slot.0,
        parent_entity.generation.0,
    );
    let inherited = rows
        .iter()
        .find(|r| r.logical_key == key)
        .expect("inherited parent_branch entity retained in child_branch");
    assert_eq!(inherited.history_versions.len(), 2);
    assert!(
        inherited.history_versions[0]
            .windows(9)
            .any(|w| w == b"parent-v1"),
        "ancestor commit 1 on parent_branch must replay exact 'parent-v1' payload"
    );
    assert!(
        inherited.history_versions[1]
            .windows(9)
            .any(|w| w == b"parent-v2"),
        "ancestor commit 2 on parent_branch must replay exact 'parent-v2' payload"
    );
    assert!(inherited.payload.windows(9).any(|w| w == b"parent-v2"));
    drop(store);
    close_durable_home(home, &request);
}

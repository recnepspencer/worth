//! Store M4.4 `S5` divergent-branch ancestry capture court:
//! proves that retaining forked Relational branches (`base -> branch_a` updating
//! aspect A, `base -> branch_b` updating aspect B and creating a sibling relation)
//! replays every commit strictly from its own ancestry without leaking unrelated
//! sibling-branch mutations. (Multi-parent DAG replay ordering is covered by
//! `worth-store-application::historical_cursor::relational_capture_replay_tests`.)

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_runtime_world::facade::{
    CanonicalKeyDomain, HistoricalBasisRetentionKind, HistoricalQueryContract,
    ORDERING_CANONICAL_ASC, PROJECTION_CANONICAL_FULL,
};
use worth_store_application::HistoricalChunkGeometry;

use super::s5_relational_diamond_fixture::{
    live_ids, open_diamond_home, seed_entity, seed_relation, update_entity_aspect,
    update_relation_aspect,
};
use super::{
    canonical_logical_key, close_durable_home, commit_and_release, drain_cursor, test_budget,
};

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn s5_divergent_sibling_branches_isolate_history_for_entities_and_relations() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let branch_a = BranchId("main".into());
    let branch_b = BranchId("sibling-b".into());

    let home = open_diamond_home(dir.path(), &execution);
    let runtime = home.relational().runtime();

    seed_entity(runtime, &execution, &branch_a, "e0", "A-BASE", "B-BASE");
    seed_entity(runtime, &execution, &branch_a, "e1", "E1-A", "E1-B");
    let (eids, _) = live_ids(runtime, &execution, &branch_a);
    let (e0, e1) = (eids[0], eids[1]);
    seed_relation(
        runtime, &execution, &branch_a, "r0", e0, e1, "RA-BASE", "RB-BASE",
    );
    let (_, rids) = live_ids(runtime, &execution, &branch_a);
    let target_r0 = rids[0];

    // Fork `sibling-b` from base, then diverge `branch_a` and `branch_b`.
    let (_, fork_basis) = runtime.observe_fork_source(&branch_a).unwrap();
    runtime
        .fork_branch(&execution, branch_b.clone(), fork_basis)
        .unwrap();

    commit_and_release(
        runtime,
        &execution,
        &branch_a,
        update_entity_aspect(e0, "alpha", "A-BRANCH-ONLY"),
    );
    commit_and_release(
        runtime,
        &execution,
        &branch_a,
        update_relation_aspect(target_r0, "ralpha", "RA-BRANCH-ONLY"),
    );
    commit_and_release(
        runtime,
        &execution,
        &branch_b,
        update_entity_aspect(e0, "beta", "B-SIBLING-ONLY"),
    );
    seed_relation(
        runtime,
        &execution,
        &branch_b,
        "r1",
        e1,
        e0,
        "R1A-SIBLING-ONLY",
        "R1B-SIBLING-ONLY",
    );

    let store = home.historical_basis_store(budget).unwrap();
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 8,
        continuation_page_versions: 4,
        layered_initial: true,
    };
    let receipt_a = store
        .retain_relational_branch_revision(
            runtime,
            &execution,
            &branch_a,
            SchemaId("diamond.merge".into()),
            SchemaVersionId(1),
            &[KindId(1)],
            &[KindId(2)],
            HistoricalBasisRetentionKind::PinLease,
            geom,
        )
        .expect("retain branch_a revision");
    let receipt_b = store
        .retain_relational_branch_revision(
            runtime,
            &execution,
            &branch_b,
            SchemaId("diamond.merge".into()),
            SchemaVersionId(1),
            &[KindId(1)],
            &[KindId(2)],
            HistoricalBasisRetentionKind::PinLease,
            geom,
        )
        .expect("retain branch_b revision");

    let read_domain = |receipt: &worth_store_application::RetainedRevisionReceipt,
                       domain: CanonicalKeyDomain,
                       kind: KindId| {
        let q = HistoricalQueryContract::new(
            receipt.schema_version.clone(),
            domain,
            Some(kind),
            PROJECTION_CANONICAL_FULL,
            ORDERING_CANONICAL_ASC,
        );
        let mut cur = store
            .open_historical_cursor(
                &execution,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                q,
                &receipt.schema_version,
            )
            .unwrap();
        drain_cursor(&mut cur, &execution, 4)
    };

    let e_key = canonical_logical_key(1, e0.partition_id.0, e0.local_slot.0, e0.generation.0);
    let r0_key = canonical_logical_key(
        2,
        target_r0.partition_id.0,
        target_r0.local_slot.0,
        target_r0.generation.0,
    );

    // Branch A must carry base + A-only history and zero sibling B mutations.
    let ent_a = read_domain(&receipt_a, CanonicalKeyDomain::Entity, KindId(1));
    let e_row_a = ent_a
        .iter()
        .find(|r| r.logical_key == e_key)
        .expect("e0 on A");
    assert_eq!(e_row_a.history_versions.len(), 2);
    assert!(
        contains_bytes(&e_row_a.history_versions[0], b"A-BASE")
            && contains_bytes(&e_row_a.history_versions[0], b"B-BASE")
    );
    assert!(
        contains_bytes(&e_row_a.history_versions[1], b"A-BRANCH-ONLY")
            && contains_bytes(&e_row_a.history_versions[1], b"B-BASE")
            && !contains_bytes(&e_row_a.history_versions[1], b"B-SIBLING-ONLY")
    );
    let rel_a = read_domain(&receipt_a, CanonicalKeyDomain::Relation, KindId(2));
    assert_eq!(rel_a.len(), 1, "branch A must not see sibling-created r1");
    assert_eq!(rel_a[0].logical_key, r0_key);
    assert_eq!(rel_a[0].history_versions.len(), 2);
    assert!(contains_bytes(
        &rel_a[0].history_versions[1],
        b"RA-BRANCH-ONLY"
    ));

    // Branch B must carry base + B-only history and zero sibling A mutations.
    let ent_b = read_domain(&receipt_b, CanonicalKeyDomain::Entity, KindId(1));
    let e_row_b = ent_b
        .iter()
        .find(|r| r.logical_key == e_key)
        .expect("e0 on B");
    assert_eq!(e_row_b.history_versions.len(), 2);
    assert!(
        contains_bytes(&e_row_b.history_versions[1], b"A-BASE")
            && contains_bytes(&e_row_b.history_versions[1], b"B-SIBLING-ONLY")
            && !contains_bytes(&e_row_b.history_versions[1], b"A-BRANCH-ONLY")
    );
    let rel_b = read_domain(&receipt_b, CanonicalKeyDomain::Relation, KindId(2));
    assert_eq!(rel_b.len(), 2, "branch B sees r0 and sibling-created r1");
    let r1_row = rel_b
        .iter()
        .find(|r| r.logical_key != r0_key)
        .expect("sibling-created relation r1");
    assert!(
        contains_bytes(&r1_row.history_versions[0], b"R1A-SIBLING-ONLY")
            && contains_bytes(&r1_row.history_versions[0], b"R1B-SIBLING-ONLY")
            && !contains_bytes(&r1_row.history_versions[0], b"RA-BRANCH-ONLY")
    );

    drop(store);
    close_durable_home(home, &request);
}

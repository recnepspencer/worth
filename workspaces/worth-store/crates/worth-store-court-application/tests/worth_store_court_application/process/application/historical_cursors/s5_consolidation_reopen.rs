//! Store M4.4 `S5` consolidation-invariance, admitted projection/ordering
//! contract enforcement, and reopen/restart certification courts on BOTH
//! Directory and Memory backends.

use worth_execution::report::{Disposition, Reportable};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::KindId;
use worth_runtime_world::facade::{
    BranchOriginIdentity, CanonicalKeyDomain, HistoricalBasisRetentionKind,
    HistoricalQueryContract, HistoricalReadDenial, ORDERING_CANONICAL_ASC,
    PROJECTION_CANONICAL_FULL, PROJECTION_CANONICAL_HEAD_ONLY, PROJECTION_CANONICAL_KEYS_ONLY,
    RETAINED_BASIS_TOKEN_PREFIX,
};
use worth_store_application::{
    HistoricalChunkGeometry, MemoryHistoricalBasisStore, RetainedBasisToken,
};

use super::{
    close_durable_home, drain_cursor, make_rows, open_durable_home, test_budget, test_origin,
    test_query, test_query_with_projection, test_schema,
};

#[test]
fn s5_directory_pre_consolidation_token_resumes_after_consolidation_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0x41);
    let query = test_query(&schema);
    let c1 = CommitId(101);
    let c2 = CommitId(102);
    let c1_rows = make_rows(40, "commit-c1", 10, 64);
    let c2_rows = make_rows(40, "commit-head-c2", 3, 64);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 8,
        inline_history_limit: 3,
        continuation_page_versions: 4,
        layered_initial: true,
    };

    let (home_id, first_incarnation, c1_commitment, pre_locators, page1_rows, wire_token) = {
        let home = open_durable_home(dir.path(), &request);
        let (_, observed) = home
            .relational()
            .runtime()
            .observe_branch(&home.relational().runtime().main_branch_identity())
            .expect("observe main branch");
        let basis_descriptor = observed.descriptor();
        let origin = BranchOriginIdentity::from_basis_descriptor(&basis_descriptor)
            .expect("derive branch origin from RelationalBranchBasisDescriptor");
        drop(observed);

        let store = home
            .historical_basis_store(budget)
            .expect("open durable historical basis store");
        let (c1_comm, _c1_pin) = store
            .retain_basis(
                c1,
                origin.clone(),
                schema.clone(),
                HistoricalBasisRetentionKind::PinLease,
                c1_rows.clone(),
                geom,
            )
            .expect("retain C1");
        let (_epoch0, pre_locators) = store.basis_locator_summary(c1).expect("C1 locators");

        let mut cursor = store
            .open_historical_cursor(&execution, c1, c1_comm, query.clone(), &schema)
            .expect("open C1 cursor");
        assert!(cursor.loan_active());
        let page1 = cursor.next_page(&execution, 11).expect("read page 1");
        assert_eq!(page1.rows().len(), 11);
        assert!(page1.has_more());
        let wire = page1.checkpoint_token().encode_wire();
        assert!(wire.starts_with(RETAINED_BASIS_TOKEN_PREFIX));
        assert_eq!(
            cursor.peek_borrowed_row().expect("borrowed row").get(),
            page1.rows().last().unwrap()
        );
        drop(cursor);

        // Advance HEAD to C2 with distinct payloads.
        store
            .retain_basis(
                c2,
                origin,
                schema.clone(),
                HistoricalBasisRetentionKind::LiveBranch,
                c2_rows,
                geom,
            )
            .expect("retain HEAD C2");

        // Consolidate C1: every physical chunk locator MUST change while commitment stays identical.
        let (post_comm, _new_pin) = store.consolidate_basis(c1).expect("consolidate C1");
        assert_eq!(post_comm, c1_comm);
        let (epoch1, post_locators) = store.basis_locator_summary(c1).expect("post locators");
        assert_eq!(epoch1, 1);
        assert_eq!(pre_locators.len(), post_locators.len());
        assert!(
            pre_locators
                .iter()
                .zip(post_locators.iter())
                .all(|(before, after)| before != after),
            "every physical chunk locator must change after layer consolidation"
        );

        let home_id = home.home_identity();
        let inc = home.incarnation();
        drop(store);
        close_durable_home(home, &request);
        (home_id, inc, c1_comm, pre_locators, page1.into_rows(), wire)
    };

    // Reopen Directory Home -> HomeIncarnation increments while HomeIdentity & MAC key persist.
    let reopened = open_durable_home(dir.path(), &request);
    assert_eq!(reopened.home_identity(), home_id);
    assert!(reopened.incarnation() > first_incarnation);
    let reopened_store = reopened
        .historical_basis_store(budget)
        .expect("restore historical basis store");
    let (epoch_after, locators_after) = reopened_store
        .basis_locator_summary(c1)
        .expect("C1 survives reopen");
    assert_eq!(epoch_after, 1);
    assert_ne!(locators_after, pre_locators);

    let decoded =
        RetainedBasisToken::decode_wire(&wire_token).expect("decode pre-consolidation token");
    assert_eq!(*decoded.root_semantic_commitment(), c1_commitment);
    let mut resumed = reopened_store
        .resume_cursor_wire(&execution, &wire_token, query, &schema)
        .expect("resume pre-consolidation wire token after consolidation + reopen");
    let mut combined = page1_rows;
    combined.extend(drain_cursor(&mut resumed, &execution, 7));
    assert_eq!(
        combined, c1_rows,
        "exact C1 rows with zero duplicates, gaps, or HEAD rows"
    );
    drop(resumed);
    drop(reopened_store);
    close_durable_home(reopened, &request);
}

#[test]
fn s5_memory_pre_consolidation_token_resumes_after_consolidation_and_restart() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let home_id = HomeIdentity::from_observed_bytes([0x22; 16]).expect("non-zero");
    let store = MemoryHistoricalBasisStore::new(home_id, HomeIncarnation::initial(), budget);
    let schema = test_schema(2, 0x52);
    let query = test_query(&schema);
    let origin = test_origin("main", 0x33);
    let c1 = CommitId(201);
    let c2 = CommitId(202);
    let c1_rows = make_rows(36, "mem-c1", 9, 48);
    let c2_rows = make_rows(36, "mem-head-c2", 2, 48);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 6,
        inline_history_limit: 2,
        continuation_page_versions: 3,
        layered_initial: true,
    };

    let (c1_comm, _) = store
        .retain_basis(
            c1,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::ArchivedBranch,
            c1_rows.clone(),
            geom,
        )
        .expect("retain C1");
    let (_, pre_locators) = store.basis_locator_summary(c1).unwrap();

    let mut cursor = store
        .open_historical_cursor(&execution, c1, c1_comm, query.clone(), &schema)
        .expect("open C1");
    let page1 = cursor.next_page(&execution, 10).expect("page 1");
    let wire = page1.checkpoint_token().encode_wire();
    drop(cursor);

    store
        .retain_basis(
            c2,
            origin,
            schema.clone(),
            HistoricalBasisRetentionKind::LiveBranch,
            c2_rows,
            geom,
        )
        .expect("retain HEAD C2");
    let (post_comm, _) = store.consolidate_basis(c1).expect("consolidate C1");
    assert_eq!(post_comm, c1_comm);
    let (_, post_locators) = store.basis_locator_summary(c1).unwrap();
    assert!(pre_locators.iter().zip(&post_locators).all(|(a, b)| a != b));

    let restarted = store.simulate_restart(budget);
    assert!(restarted.home_incarnation() > store.home_incarnation());
    let mut resumed = restarted
        .resume_cursor_wire(&execution, &wire, query, &schema)
        .expect("resume wire token after consolidation + restart");
    let mut combined = page1.into_rows();
    combined.extend(drain_cursor(&mut resumed, &execution, 5));
    assert_eq!(combined, c1_rows);
}

#[test]
fn s5_admitted_projections_and_unadmitted_contract_refusal() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0x44);
    let origin = test_origin("main", 0x45);
    let cid = CommitId(250);
    let rows = make_rows(16, "proj", 8, 48);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };

    let store = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0x46; 16]).unwrap(),
        HomeIncarnation::initial(),
        budget,
    );
    let (comm, _) = store
        .retain_basis(
            cid,
            origin,
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            rows.clone(),
            geom,
        )
        .unwrap();

    // 1. KeysOnly reads ZERO continuation pages and zeroes payload + history_versions.
    let q_keys = test_query_with_projection(&schema, PROJECTION_CANONICAL_KEYS_ONLY);
    let mut c_keys = store
        .open_historical_cursor(&execution, cid, comm, q_keys, &schema)
        .unwrap();
    let keys_out = drain_cursor(&mut c_keys, &execution, 5);
    assert_eq!(keys_out.len(), rows.len());
    assert!(keys_out
        .iter()
        .all(|r| r.payload.is_empty() && r.history_versions.is_empty()));
    assert_eq!(store.residency_snapshot().continuation_pages_faulted, 0);

    // 2. HeadOnly emits payloads while reading ZERO continuation pages.
    let q_head = test_query_with_projection(&schema, PROJECTION_CANONICAL_HEAD_ONLY);
    let mut c_head = store
        .open_historical_cursor(&execution, cid, comm, q_head, &schema)
        .unwrap();
    let head_out = drain_cursor(&mut c_head, &execution, 5);
    assert_eq!(head_out.len(), rows.len());
    assert!(head_out
        .iter()
        .zip(&rows)
        .all(|(a, b)| a.payload == b.payload && a.history_versions.is_empty()));
    assert_eq!(store.residency_snapshot().continuation_pages_faulted, 0);

    // 3. Full emits complete payloads + all spilled history versions.
    let q_full = test_query_with_projection(&schema, PROJECTION_CANONICAL_FULL);
    let mut c_full = store
        .open_historical_cursor(&execution, cid, comm, q_full, &schema)
        .unwrap();
    let full_out = drain_cursor(&mut c_full, &execution, 5);
    assert_eq!(full_out, rows);
    assert!(store.residency_snapshot().continuation_pages_faulted > 0);

    // 4. Unadmitted projection name and unadmitted ordering MUST refuse explicitly.
    let bad_proj = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(7)),
        "custom.unadmitted.proj",
        ORDERING_CANONICAL_ASC,
    );
    let err_proj = store
        .open_historical_cursor(&execution, cid, comm, bad_proj, &schema)
        .err()
        .unwrap();
    assert!(matches!(
        err_proj,
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    assert_eq!(err_proj.disposition(), Disposition::CorrectInput);

    let bad_ord = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(7)),
        PROJECTION_CANONICAL_FULL,
        "key.desc.unadmitted",
    );
    let err_ord = store
        .open_historical_cursor(&execution, cid, comm, bad_ord, &schema)
        .err()
        .unwrap();
    assert!(matches!(
        err_ord,
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    assert_eq!(err_ord.disposition(), Disposition::CorrectInput);
}

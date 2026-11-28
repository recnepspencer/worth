//! Store M4.4 `S5` real Relational revision certification court: commits multi-update
//! entities + a relation to live `RelationalRuntime`, compares resumed `Full` canonical
//! binary payloads (`WHRE1`/`WHRR1`) and real multi-version history (`WHRV1`) bit-for-bit
//! against independent pre-retain Relational ground truth across consolidation and reopen.

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, KindId, VersionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_runtime_world::facade::{
    CanonicalKeyDomain, HistoricalBasisRetentionKind, HistoricalBasisRow, HistoricalQueryContract,
    HistoricalReadDenial, ORDERING_CANONICAL_ASC, PROJECTION_CANONICAL_FULL,
};
use worth_store_application::{
    encode_canonical_entity_payload, encode_canonical_relation_payload, HistoricalChunkGeometry,
};

use super::{
    build_history_versions, canonical_logical_key, close_durable_home, commit_and_release,
    drain_cursor, open_durable_home, test_budget,
};

#[test]
fn s5_real_relational_branch_revision_preserves_canonical_payloads_and_multi_version_history() {
    let dir = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let main_branch = BranchId("main".into());
    let schema_id = SchemaId("m4.relational.revision.s5".into());
    let schema_ver = SchemaVersionId(1);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 2,
        inline_history_limit: 1,
        continuation_page_versions: 1,
        layered_initial: true,
    };

    let (
        receipt,
        ground_truth_entities,
        ground_truth_relations,
        page1_rows,
        wire_token,
        ent_query,
        rel_query,
    ) = {
        let home = open_durable_home(dir.path(), &request);
        let runtime = home.relational().runtime();

        // 1. Create 4 entities across 4 commits.
        for step in 0..4usize {
            commit_and_release(
                runtime,
                &execution,
                &main_branch,
                crate::batches::create(step),
            );
        }

        // Read live entity IDs directly from Relational observation.
        let main_id = runtime.branch_identity(&main_branch).unwrap();
        let (_, basis_init) = runtime.observe_branch(&main_id).unwrap();
        let init_view = runtime
            .read_truth()
            .read_observation(&execution, &basis_init.observation())
            .unwrap()
            .unwrap();
        let mut created_entities: Vec<EntityId> = init_view
            .entities()
            .iter()
            .filter(|e| e.lifecycle == RecordLifecycleState::Live && e.kind.kind_id == KindId(1))
            .map(|e| e.entity_id)
            .collect();
        created_entities.sort();
        assert_eq!(created_entities.len(), 4);
        drop(basis_init);

        // 2. Commit TWO distinct updates on entity[0], ONE update on entity[1], and ONE relation!
        let target_entity = created_entities[0];
        for batch in [
            crate::batches::update(target_entity, 101),
            crate::batches::update(target_entity, 102),
            crate::batches::update(created_entities[1], 103),
            crate::batches::relate(created_entities[0], created_entities[1], 200),
        ] {
            commit_and_release(runtime, &execution, &main_branch, batch);
        }
        runtime
            .durability_authority()
            .checkpoint(&execution)
            .expect("checkpoint V1");

        // 3. Build independent pre-retain Relational ground truth BEFORE retaining!
        let (_, basis_v1) = runtime.observe_branch(&main_id).unwrap();
        let v1_ver = VersionId(basis_v1.descriptor().truth_version().as_u64());
        let v1_view = runtime
            .read_truth()
            .read_observation(&execution, &basis_v1.observation())
            .unwrap()
            .unwrap();
        let mut expected_entities = Vec::new();
        for rec in v1_view.entities() {
            if rec.lifecycle != RecordLifecycleState::Live || rec.kind.kind_id != KindId(1) {
                continue;
            }
            let key = canonical_logical_key(
                rec.kind.kind_id.0,
                rec.entity_id.partition_id.0,
                rec.entity_id.local_slot.0,
                rec.entity_id.generation.0,
            );
            let payload = encode_canonical_entity_payload(rec.authoritative_aspect_state.as_ref());
            let entries = runtime
                .history()
                .entity_aspect_history(&execution, &main_branch, rec.entity_id, None)
                .unwrap();
            let history_versions = build_history_versions(entries, v1_ver, &payload, |vid| {
                let hr = runtime
                    .read_truth()
                    .visible_entity_at_version(&execution, rec.entity_id, vid)
                    .unwrap();
                encode_canonical_entity_payload(
                    hr.as_ref()
                        .and_then(|r| r.authoritative_aspect_state.as_ref()),
                )
            });
            expected_entities.push(HistoricalBasisRow {
                partition_id: rec.entity_id.partition_id,
                domain: CanonicalKeyDomain::Entity,
                kind_id: rec.kind.kind_id,
                logical_key: key,
                payload,
                history_versions,
            });
        }
        expected_entities.sort_by(|a, b| a.canonical_cursor_key().cmp(&b.canonical_cursor_key()));

        let mut expected_relations = Vec::new();
        for rec in v1_view.relations() {
            if rec.lifecycle != RecordLifecycleState::Live || rec.kind.kind_id != KindId(2) {
                continue;
            }
            let key = canonical_logical_key(
                rec.kind.kind_id.0,
                rec.relation_id.partition_id.0,
                rec.relation_id.local_slot.0,
                rec.relation_id.generation.0,
            );
            let payload = encode_canonical_relation_payload(
                rec.source,
                rec.target,
                rec.authoritative_aspect_state.as_ref(),
            );
            let entries = runtime
                .history()
                .relation_aspect_history(&execution, &main_branch, rec.relation_id, None)
                .unwrap();
            let history_versions = build_history_versions(entries, v1_ver, &payload, |vid| {
                let hr = runtime
                    .read_truth()
                    .visible_relation_at_version(&execution, rec.relation_id, vid)
                    .unwrap();
                encode_canonical_relation_payload(
                    rec.source,
                    rec.target,
                    hr.as_ref()
                        .and_then(|r| r.authoritative_aspect_state.as_ref()),
                )
            });
            expected_relations.push(HistoricalBasisRow {
                partition_id: rec.relation_id.partition_id,
                domain: CanonicalKeyDomain::Relation,
                kind_id: rec.kind.kind_id,
                logical_key: key,
                payload,
                history_versions,
            });
        }
        expected_relations.sort_by(|a, b| a.canonical_cursor_key().cmp(&b.canonical_cursor_key()));
        drop(basis_v1);

        assert!(expected_entities[0].history_versions.len() >= 3);
        assert_eq!(expected_relations.len(), 1);
        assert!(expected_entities[0].payload.starts_with(b"WHRE1"));
        assert!(expected_entities[0]
            .payload
            .windows(11)
            .any(|w| w == b"renamed-102"));
        assert!(expected_entities[0]
            .history_versions
            .iter()
            .all(|h| h.starts_with(b"WHRV1")));
        assert!(expected_relations[0].payload.starts_with(b"WHRR1"));
        assert!(expected_relations[0]
            .payload
            .windows(8)
            .any(|w| w == b"link-200"));

        // 4. Retain V1 directly from live RelationalRuntime.
        let store = home.historical_basis_store(budget).unwrap();
        let receipt = store
            .retain_relational_branch_revision(
                runtime,
                &execution,
                &main_branch,
                schema_id,
                schema_ver,
                &[KindId(1)],
                &[KindId(2)],
                HistoricalBasisRetentionKind::PinLease,
                geom,
            )
            .expect("retain real committed Relational revision V1");
        let ent_query = HistoricalQueryContract::new(
            receipt.schema_version.clone(),
            CanonicalKeyDomain::Entity,
            Some(KindId(1)),
            PROJECTION_CANONICAL_FULL,
            ORDERING_CANONICAL_ASC,
        );
        let rel_query = HistoricalQueryContract::new(
            receipt.schema_version.clone(),
            CanonicalKeyDomain::Relation,
            Some(KindId(2)),
            PROJECTION_CANONICAL_FULL,
            ORDERING_CANONICAL_ASC,
        );

        let mut cur = store
            .open_historical_cursor(
                &execution,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                ent_query.clone(),
                &receipt.schema_version,
            )
            .unwrap();
        let p1 = cur.next_page(&execution, 2).expect("read page 1");
        assert!(p1.rows().len() == 2 && p1.has_more());
        let wire = p1.checkpoint_token().encode_wire();
        let page1_rows = p1.into_rows();
        drop(cur);

        // 5. Advance `main` HEAD to State V2: overwrite target_entity AND create 4 new entities!
        commit_and_release(
            runtime,
            &execution,
            &main_branch,
            crate::batches::update(target_entity, 999),
        );
        for step in 4..8usize {
            commit_and_release(
                runtime,
                &execution,
                &main_branch,
                crate::batches::create(step),
            );
        }
        runtime
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        let (post_comm, _) = store.consolidate_basis(receipt.commit_id).unwrap();
        assert_eq!(post_comm, receipt.root_semantic_commitment);
        drop(store);
        close_durable_home(home, &request);
        (
            receipt,
            expected_entities,
            expected_relations,
            page1_rows,
            wire,
            ent_query,
            rel_query,
        )
    };

    // 6. Cold reopen: resumed V1 MUST match independent pre-retain Relational ground truth bit-for-bit!
    {
        let reopened = open_durable_home(dir.path(), &request);
        let store = reopened.historical_basis_store(budget).unwrap();
        let mut resumed = store
            .resume_cursor_wire(
                &execution,
                &wire_token,
                ent_query.clone(),
                &receipt.schema_version,
            )
            .expect("resume real Relational V1 revision after consolidation + reopen");
        let mut combined = page1_rows;
        combined.extend(drain_cursor(&mut resumed, &execution, 2));
        assert_eq!(combined, ground_truth_entities);
        for row in &combined {
            assert!(!row.payload.windows(11).any(|w| w == b"renamed-999"));
            assert!(row
                .history_versions
                .iter()
                .all(|h| h.starts_with(b"WHRV1") && !h.windows(11).any(|w| w == b"renamed-999")));
        }
        drop(resumed);

        let mut rel_cur = store
            .open_historical_cursor(
                &execution,
                receipt.commit_id,
                receipt.root_semantic_commitment,
                rel_query,
                &receipt.schema_version,
            )
            .unwrap();
        assert_eq!(
            drain_cursor(&mut rel_cur, &execution, 2),
            ground_truth_relations
        );
        drop(rel_cur);
        store.prune_basis(receipt.commit_id).expect("prune V1");
        drop(store);
        close_durable_home(reopened, &request);
    }

    // 7. Reopen after prune: resuming wire token MUST refuse HistoryUnavailable (never HEAD!).
    {
        let reopened = open_durable_home(dir.path(), &request);
        let store = reopened.historical_basis_store(budget).unwrap();
        let err = store
            .resume_cursor_wire(&execution, &wire_token, ent_query, &receipt.schema_version)
            .err()
            .expect("pruned Relational V1 revision must refuse after reopen");
        assert_eq!(
            err,
            HistoricalReadDenial::HistoryUnavailable {
                commit_id: receipt.commit_id
            }
        );
        let _ = CommitId(0);
        drop(store);
        close_durable_home(reopened, &request);
    }
}

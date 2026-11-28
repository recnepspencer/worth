//! Unit tests for crate-sealed `RetainedBasisToken` Home-MAC wire codec and
//! atomic whole-page commit in `HistoricalCursor::next_page`.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};
use worth_runtime_world::facade::{
    BranchOriginIdentity, CanonicalCursorKey, CanonicalKeyDomain, HistoricalQueryContract,
    HistoricalReadDenial, RetainedBasisTokenDecodeError, RetainedBasisTokenFields,
    SchemaVersionIdentity, ORDERING_CANONICAL_ASC, PROJECTION_CANONICAL_FULL,
    RETAINED_BASIS_TOKEN_PREFIX,
};
use worth_store::pin_table::{RootPinDropPolicy, VolatileRootPinTable};

use super::authority::HistoricalCursorAuthority;
use super::cursor::StoreLoanCustody;
use super::memory_support::MemoryLoanCustody;
use super::token::RetainedBasisToken;

fn sample_token(authority: &HistoricalCursorAuthority) -> RetainedBasisToken {
    let schema = SchemaVersionIdentity::new(
        SchemaId("audit.schema".into()),
        SchemaVersionId(3),
        [0x5Au8; 32],
    )
    .expect("valid schema");
    let query = HistoricalQueryContract::new(
        schema.clone(),
        CanonicalKeyDomain::Entity,
        Some(KindId(7)),
        PROJECTION_CANONICAL_FULL,
        ORDERING_CANONICAL_ASC,
    );
    let origin = BranchOriginIdentity::from_owner(BranchId("main".into()), [0x3Cu8; 32])
        .expect("valid origin");
    let pos = CanonicalCursorKey::after_entity_key(PartitionId(2), KindId(7), b"entry-0042")
        .expect("valid key");
    authority
        .issue_token(RetainedBasisTokenFields {
            home_identity: HomeIdentity::from_observed_bytes([0x11; 16]).expect("non-zero"),
            home_incarnation: HomeIncarnation::initial(),
            branch_origin: origin,
            commit_id: CommitId(19),
            root_semantic_commitment: [0xAB; 32],
            schema_version: schema,
            query_digest: query.query_digest(),
            canonical_key_position: pos,
        })
        .expect("sealed token")
}

#[test]
fn token_wire_codec_round_trips_and_enforces_home_mac_and_size_bounds() {
    let auth = HistoricalCursorAuthority::from_home_key([0x42; 32]);
    let foreign_auth = HistoricalCursorAuthority::from_home_key([0x99; 32]);
    let token = sample_token(&auth);
    let wire = token.encode_wire();
    assert!(wire.starts_with(RETAINED_BASIS_TOKEN_PREFIX));

    let decoded = RetainedBasisToken::decode_wire(&wire).expect("valid token decodes");
    assert_eq!(decoded, token);
    assert!(auth.verify_token(&decoded));
    assert!(
        !foreign_auth.verify_token(&decoded),
        "token minted by one Home must fail MAC verification on another Home"
    );

    assert_eq!(
        RetainedBasisToken::decode_wire(&format!("whc2_{}", &wire[5..])),
        Err(RetainedBasisTokenDecodeError::InvalidPrefix)
    );
    let huge = format!("{RETAINED_BASIS_TOKEN_PREFIX}{}", "A".repeat(20_000));
    assert_eq!(
        RetainedBasisToken::decode_wire(&huge),
        Err(RetainedBasisTokenDecodeError::MalformedField)
    );

    let same = token.canonical_key_position().clone();
    assert!(auth.advance_token(&token, same).is_none());
    let next = CanonicalCursorKey::after_entity_key(PartitionId(2), KindId(7), b"entry-0043")
        .expect("valid key");
    let advanced = auth
        .advance_token(&token, next.clone())
        .expect("strictly greater");
    assert_eq!(advanced.canonical_key_position(), &next);
    assert!(auth.verify_token(&advanced));
    assert_eq!(
        RetainedBasisToken::decode_wire(&advanced.encode_wire()).unwrap(),
        advanced
    );
}

#[test]
fn next_page_leaves_cursor_token_unchanged_on_mid_page_order_violation() {
    let auth = HistoricalCursorAuthority::from_home_key([0x55; 32]);
    let initial = sample_token(&auth);
    let pins = Arc::new(VolatileRootPinTable::default());
    let pin = pins
        .pin(1, None, RootPinDropPolicy::RetainUntilChildrenDrain, vec![])
        .unwrap();
    pins.loan(pin).unwrap();
    let custody = StoreLoanCustody::Memory(MemoryLoanCustody { pins, pin });

    let k1 =
        CanonicalCursorKey::after_entity_key(PartitionId(2), KindId(7), b"entry-0050").expect("k1");
    let k_bad = CanonicalCursorKey::after_entity_key(PartitionId(2), KindId(7), b"entry-0045")
        .expect("k_bad < k1");
    let mut cursor = auth.admit_cursor(
        initial.clone(),
        custody,
        Arc::new(AtomicBool::new(true)),
        Box::new(move |_exec, _after, _limit| {
            Ok((vec![(k1.clone(), 1u32), (k_bad.clone(), 2u32)], true))
        }),
    );

    let req = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&req);
    let err = cursor.next_page(&exec, 2).unwrap_err();
    assert!(matches!(err, HistoricalReadDenial::CorruptStorage { .. }));
    assert_eq!(
        cursor.checkpoint_token(),
        initial,
        "mid-page order violation must NOT advance cursor token"
    );
    assert!(cursor.peek_borrowed_row().is_none());
}

#[test]
fn chunk_scan_enforces_streaming_window_bytes_on_borrowed_slices_before_owning_rows() {
    use worth_runtime_world::facade::{
        AdmittedHistoricalProjection, HistoricalBasisRow, ORDERING_CANONICAL_ASC,
        PROJECTION_CANONICAL_FULL,
    };

    use super::chunk_format::{seal_basis_chunks, HistoricalChunkGeometry};
    use super::chunk_scan::scan_verified_chunk_bounded;

    let schema = SchemaVersionIdentity::new(
        SchemaId("chunk.window.test".into()),
        SchemaVersionId(1),
        [0x77; 32],
    )
    .unwrap();
    let query = HistoricalQueryContract::new(
        schema,
        CanonicalKeyDomain::Entity,
        Some(KindId(7)),
        PROJECTION_CANONICAL_FULL,
        ORDERING_CANONICAL_ASC,
    );
    // Row 0 is large (4096B payload), row 1 is small (64B), row 2 is oversized (2048B).
    let rows = vec![
        HistoricalBasisRow {
            partition_id: PartitionId(0),
            domain: CanonicalKeyDomain::Entity,
            kind_id: KindId(7),
            logical_key: b"k-0".to_vec(),
            payload: vec![b'a'; 4096],
            history_versions: vec![vec![b'h'; 256]],
        },
        HistoricalBasisRow {
            partition_id: PartitionId(0),
            domain: CanonicalKeyDomain::Entity,
            kind_id: KindId(7),
            logical_key: b"k-1".to_vec(),
            payload: vec![b'b'; 64],
            history_versions: vec![vec![b'i'; 32]],
        },
        HistoricalBasisRow {
            partition_id: PartitionId(0),
            domain: CanonicalKeyDomain::Entity,
            kind_id: KindId(7),
            logical_key: b"k-2".to_vec(),
            payload: vec![b'c'; 2048],
            history_versions: vec![vec![b'j'; 32]],
        },
    ];
    let sealed = seal_basis_chunks(
        rows,
        HistoricalChunkGeometry {
            rows_per_chunk: 8,
            inline_history_limit: 4,
            continuation_page_versions: 4,
            layered_initial: true,
        },
    )
    .unwrap();
    let desc = &sealed.chunks[0];
    let chunk_bytes = sealed.members.get(&desc.chunk_member).unwrap();

    // Resuming strictly AFTER oversized row 0 under a 256-byte window MUST skip row 0
    // without allocating it, return row 1, and stop cleanly before oversized row 2.
    let after_row0 =
        CanonicalCursorKey::after(PartitionId(0), CanonicalKeyDomain::Entity, b"k-0".to_vec())
            .unwrap();
    let mut acc = 0usize;
    let mut collected = Vec::new();
    let has_more = scan_verified_chunk_bounded(
        desc,
        chunk_bytes,
        &after_row0,
        &query,
        AdmittedHistoricalProjection::Full,
        8,
        256,
        &mut acc,
        &mut collected,
        |_, _| panic!("must not fault continuations"),
    )
    .expect("skips row 0 and returns row 1 within 256B window");
    assert!(has_more);
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].1.logical_key, b"k-1");

    // Resuming after row 1 hits oversized row 2 first -> refuses with MemoryExhausted!
    let after_row1 = collected[0].0.clone();
    let mut acc2 = 0usize;
    let mut collected2 = Vec::new();
    let err = scan_verified_chunk_bounded(
        desc,
        chunk_bytes,
        &after_row1,
        &query,
        AdmittedHistoricalProjection::Full,
        8,
        256,
        &mut acc2,
        &mut collected2,
        |_, _| panic!("must refuse before faulting continuations"),
    )
    .unwrap_err();
    assert!(matches!(err, HistoricalReadDenial::MemoryExhausted { .. }));
    assert!(collected2.is_empty());
}

#[test]
fn retained_branch_index_tracks_refcounts_across_retain_prune_and_memory_store() {
    use worth_relational::facade::config::ResidencyBudget;
    use worth_relational::facade::identity::VersionId;
    use worth_runtime_world::facade::HistoricalBasisRetentionKind;

    use super::catalog::RetainedBranchIndex;
    use super::memory::MemoryHistoricalBasisStore;

    let mut idx = RetainedBranchIndex::default();
    assert!(!idx.contains("b-1"));
    idx.increment(Some("b-1"));
    idx.increment(Some("b-1"));
    assert!(idx.contains("b-1"));
    idx.decrement(Some("b-1"));
    assert!(idx.contains("b-1"), "refcount=1 still retained");
    idx.decrement(Some("b-1"));
    assert!(!idx.contains("b-1"), "refcount=0 removed from index");

    let store = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0x22; 16]).unwrap(),
        HomeIncarnation::initial(),
        ResidencyBudget {
            head_working_set_bytes: 64 * 1024,
            historical_working_set_bytes: 64 * 1024,
            max_in_flight_fill_bytes: 32 * 1024,
            streaming_window_bytes: 16 * 1024,
        },
    );
    let schema = SchemaVersionIdentity::new(
        SchemaId("branch.index.test".into()),
        SchemaVersionId(1),
        [0x88; 32],
    )
    .unwrap();
    let origin = BranchOriginIdentity::from_owner(BranchId("main".into()), [0x11; 32]).unwrap();
    store
        .retain_query_branch_basis(
            CommitId(1),
            "retained-b1".into(),
            VersionId(1),
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            [0xA1; 32],
        )
        .unwrap();
    assert!(store.is_retained_branch("retained-b1"));
    assert!(!store.is_retained_branch("other"));
    // Duplicate retain of same commit keeps refcount balanced.
    store
        .retain_query_branch_basis(
            CommitId(1),
            "retained-b1".into(),
            VersionId(1),
            origin,
            schema,
            HistoricalBasisRetentionKind::PinLease,
            [0xA1; 32],
        )
        .unwrap();
    assert!(store.is_retained_branch("retained-b1"));
    store.prune_basis(CommitId(1)).unwrap();
    assert!(
        !store.is_retained_branch("retained-b1"),
        "pruning commit decrements retained branch index to zero"
    );
}

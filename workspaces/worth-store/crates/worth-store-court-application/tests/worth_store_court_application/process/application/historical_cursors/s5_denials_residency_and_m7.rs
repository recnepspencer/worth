//! Store M4.4 `S5` canonical reconstruction after auxiliary physical index discard
//! and exhaustive 5-way `HistoricalReadDenial` / `Reportable` / `Disposition` +
//! Home-MAC forgery + bounded wire decoder refusal contract.

use worth_execution::report::{
    DenialClass, DiagnosticDetail, Disposition, ReportProjection, Reportable, RetryHint,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    HistoricalBasisRetentionKind, HistoricalReadDenial, RetainedBasisTokenDecodeError,
    PROJECTION_CANONICAL_FULL, PROJECTION_CANONICAL_HEAD_ONLY, RETAINED_BASIS_TOKEN_PREFIX,
};
use worth_store_application::{
    HistoricalChunkGeometry, MemoryHistoricalBasisStore, RetainedBasisToken,
};

use super::{
    close_durable_home, drain_cursor, make_rows, open_durable_home, test_budget, test_origin,
    test_query, test_query_with_projection, test_schema,
};

#[test]
fn s5_canonical_reconstruction_after_physical_index_discard_on_directory_and_memory() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0x91);
    let query = test_query(&schema);
    let origin = test_origin("main", 0x92);
    let cid = CommitId(401);
    let rows = make_rows(48, "idx-discard", 7, 40);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 6,
        inline_history_limit: 2,
        continuation_page_versions: 3,
        layered_initial: true,
    };

    // 1. Directory backend
    let dir = tempfile::tempdir().unwrap();
    let home = open_durable_home(dir.path(), &request);
    let durable = home.historical_basis_store(budget).unwrap();
    let (comm, _) = durable
        .retain_basis(
            cid,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::ArchivedBranch,
            rows.clone(),
            geom,
        )
        .unwrap();
    assert_eq!(durable.auxiliary_index_count(), 1);
    let mut c_before = durable
        .open_historical_cursor(&execution, cid, comm, query.clone(), &schema)
        .unwrap();
    let p1 = c_before.next_page(&execution, 13).unwrap();
    let wire = p1.checkpoint_token().encode_wire();
    drop(c_before);

    assert_eq!(durable.discard_auxiliary_indexes(), 1);
    assert_eq!(durable.auxiliary_index_count(), 0);

    let mut c_after = durable
        .resume_cursor_wire(&execution, &wire, query.clone(), &schema)
        .unwrap();
    let mut combined = p1.into_rows();
    combined.extend(drain_cursor(&mut c_after, &execution, 9));
    assert_eq!(
        combined, rows,
        "Directory canonical resume after index discard must be exact"
    );
    drop(c_after);
    drop(durable);
    close_durable_home(home, &request);

    // 2. Memory backend parity
    let mem = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0x93; 16]).unwrap(),
        HomeIncarnation::initial(),
        budget,
    );
    let (comm, _) = mem
        .retain_basis(
            cid,
            origin,
            schema.clone(),
            HistoricalBasisRetentionKind::ArchivedBranch,
            rows.clone(),
            geom,
        )
        .unwrap();
    let mut c_before = mem
        .open_historical_cursor(&execution, cid, comm, query.clone(), &schema)
        .unwrap();
    let p1 = c_before.next_page(&execution, 13).unwrap();
    let wire = p1.checkpoint_token().encode_wire();
    drop(c_before);
    assert_eq!(mem.discard_auxiliary_indexes(), 1);
    assert_eq!(mem.auxiliary_index_count(), 0);
    let mut c_after = mem
        .resume_cursor_wire(&execution, &wire, query, &schema)
        .unwrap();
    let mut combined = p1.into_rows();
    combined.extend(drain_cursor(&mut c_after, &execution, 9));
    assert_eq!(
        combined, rows,
        "Memory canonical resume after index discard must be exact"
    );
}

#[test]
fn s5_exhaustive_denial_home_mac_and_bounded_decoder_contract() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema_v1 = test_schema(1, 0xA1);
    let schema_v2 = test_schema(2, 0xA2);
    let query_full = test_query_with_projection(&schema_v1, PROJECTION_CANONICAL_FULL);
    let query_head = test_query_with_projection(&schema_v1, PROJECTION_CANONICAL_HEAD_ONLY);
    let origin = test_origin("main", 0xA3);
    let cid = CommitId(501);
    let rows = make_rows(12, "denial-test", 4, 128);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };

    let dir_a = tempfile::tempdir().unwrap();
    let home_a = open_durable_home(dir_a.path(), &request);
    let durable_a = home_a.historical_basis_store(budget).unwrap();
    let (comm, _) = durable_a
        .retain_basis(
            cid,
            origin.clone(),
            schema_v1.clone(),
            HistoricalBasisRetentionKind::PinLease,
            rows.clone(),
            geom,
        )
        .unwrap();
    let mut cur = durable_a
        .open_historical_cursor(&execution, cid, comm, query_full.clone(), &schema_v1)
        .unwrap();
    let valid_token = cur
        .next_page(&execution, 2)
        .unwrap()
        .checkpoint_token()
        .clone();
    drop(cur);

    // 1. HistoryUnavailable -> Final
    let missing_err = durable_a
        .open_historical_cursor(
            &execution,
            CommitId(9999),
            comm,
            query_full.clone(),
            &schema_v1,
        )
        .err()
        .unwrap();
    assert_eq!(missing_err.disposition(), Disposition::Final);
    assert_eq!(
        missing_err
            .to_operator_report(DiagnosticDetail::Standard)
            .disposition(),
        Disposition::Final
    );

    // 2. UnsupportedHistoricalMeaning (wrong schema OR wrong query digest) -> CorrectInput
    let bad_schema = durable_a
        .resume_cursor(&execution, &valid_token, query_full.clone(), &schema_v2)
        .err()
        .unwrap();
    assert!(matches!(
        bad_schema,
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    assert_eq!(bad_schema.disposition(), Disposition::CorrectInput);

    let bad_query = durable_a
        .resume_cursor(&execution, &valid_token, query_head.clone(), &schema_v1)
        .err()
        .unwrap();
    assert!(matches!(
        bad_query,
        HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }
    ));
    assert_eq!(bad_query.disposition(), Disposition::CorrectInput);

    // 3. Cross-Home token forgery (issued by Home B with identical CommitId & commitment!) -> CorruptStorage (Escalate)
    let dir_b = tempfile::tempdir().unwrap();
    let home_b = open_durable_home(dir_b.path(), &request);
    let durable_b = home_b.historical_basis_store(budget).unwrap();
    let (comm_b, _) = durable_b
        .retain_basis(
            cid,
            origin.clone(),
            schema_v1.clone(),
            HistoricalBasisRetentionKind::PinLease,
            rows.clone(),
            geom,
        )
        .unwrap();
    assert_eq!(comm_b, comm);
    let mut cur_b = durable_b
        .open_historical_cursor(&execution, cid, comm_b, query_full.clone(), &schema_v1)
        .unwrap();
    let cross_home_wire = cur_b
        .next_page(&execution, 2)
        .unwrap()
        .checkpoint_token()
        .encode_wire();
    drop(cur_b);
    drop(durable_b);
    close_durable_home(home_b, &request);

    let cross_home_err = durable_a
        .resume_cursor_wire(&execution, &cross_home_wire, query_full.clone(), &schema_v1)
        .err()
        .expect("token issued by Home B must be rejected by Home A");
    assert!(matches!(
        cross_home_err,
        HistoricalReadDenial::CorruptStorage { .. }
    ));
    assert_eq!(cross_home_err.disposition(), Disposition::Escalate);

    // 4. Tampered wire token (flipped character in MAC/payload) -> rejected on resume
    let valid_wire = valid_token.encode_wire();
    let mut tampered = valid_wire.clone().into_bytes();
    let idx = tampered.len() - 2;
    tampered[idx] = if tampered[idx] == b'A' { b'B' } else { b'A' };
    let tampered_wire = String::from_utf8(tampered).unwrap();
    let tampered_err = durable_a
        .resume_cursor_wire(&execution, &tampered_wire, query_full.clone(), &schema_v1)
        .err()
        .expect("tampered wire token must be refused");
    assert!(matches!(
        tampered_err,
        HistoricalReadDenial::CorruptStorage { .. }
    ));

    // 5. Wire decode errors map through MalformedToken -> CorrectInput (NOT CorruptStorage/Escalate).
    assert_eq!(
        RetainedBasisToken::decode_wire(&valid_wire[..9]),
        Err(RetainedBasisTokenDecodeError::Truncated)
    );
    assert_eq!(
        RetainedBasisTokenDecodeError::Truncated.disposition(),
        Disposition::CorrectInput
    );
    let truncated_err = durable_a
        .resume_cursor_wire(&execution, &valid_wire[..9], query_full.clone(), &schema_v1)
        .err()
        .unwrap();
    assert_eq!(
        truncated_err,
        HistoricalReadDenial::MalformedToken(RetainedBasisTokenDecodeError::Truncated)
    );
    assert_eq!(truncated_err.disposition(), Disposition::CorrectInput);
    let bad_prefix_err = durable_a
        .resume_cursor_wire(
            &execution,
            "wrong_prefix_abc",
            query_full.clone(),
            &schema_v1,
        )
        .err()
        .unwrap();
    assert_eq!(
        bad_prefix_err,
        HistoricalReadDenial::MalformedToken(RetainedBasisTokenDecodeError::InvalidPrefix)
    );
    assert_eq!(bad_prefix_err.disposition(), Disposition::CorrectInput);
    let oversized = format!("{RETAINED_BASIS_TOKEN_PREFIX}{}", "A".repeat(16_384));
    assert_eq!(
        RetainedBasisToken::decode_wire(&oversized),
        Err(RetainedBasisTokenDecodeError::MalformedField)
    );
    let oversized_err = durable_a
        .resume_cursor_wire(&execution, &oversized, query_full.clone(), &schema_v1)
        .err()
        .unwrap();
    assert_eq!(
        oversized_err,
        HistoricalReadDenial::MalformedToken(RetainedBasisTokenDecodeError::MalformedField)
    );
    assert_eq!(oversized_err.disposition(), Disposition::CorrectInput);

    // 6. MemoryExhausted -> Retry(AfterBackoff) + DenialClass::Exhausted
    let _saturate = durable_a
        .borrow_historical_bytes(budget.historical_working_set_bytes)
        .expect("borrow full historical budget");
    let mut mem_cur = durable_a
        .resume_cursor(&execution, &valid_token, query_full.clone(), &schema_v1)
        .unwrap();
    let mem_err = mem_cur.next_page(&execution, 4).unwrap_err();
    assert!(matches!(
        mem_err,
        HistoricalReadDenial::MemoryExhausted { .. }
    ));
    assert_eq!(
        mem_err.disposition(),
        Disposition::Retry(RetryHint::AfterBackoff)
    );
    assert_eq!(mem_err.denial_class(), DenialClass::Exhausted);
    drop(mem_cur);
    drop(_saturate);

    // 7. RequestStopped -> Retry(Immediate) + DenialClass::Cancelled
    let cancel_source = CancellationSource::new();
    cancel_source.cancel();
    let cancelled_req =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel_source.token(), None);
    let cancelled_exec = ExecutionRequest::serial(&cancelled_req);
    let stop_err = durable_a
        .resume_cursor(
            &cancelled_exec,
            &valid_token,
            query_full.clone(),
            &schema_v1,
        )
        .err()
        .unwrap();
    assert_eq!(stop_err, HistoricalReadDenial::RequestStopped);
    assert_eq!(
        stop_err.disposition(),
        Disposition::Retry(RetryHint::Immediate)
    );
    assert_eq!(stop_err.denial_class(), DenialClass::Cancelled);

    // 8. Corrupt chunk member bytes detected on read -> CorruptStorage (Escalate)
    let (_, locators) = durable_a.basis_locator_summary(cid).unwrap();
    durable_a.inject_corrupt_member_bytes(locators[0], vec![0xFF; 32]);
    let mut corrupt_cur = durable_a
        .open_historical_cursor(&execution, cid, comm, query_full, &schema_v1)
        .unwrap();
    let corrupt_err = corrupt_cur.next_page(&execution, 4).unwrap_err();
    assert!(matches!(
        corrupt_err,
        HistoricalReadDenial::CorruptStorage { .. }
    ));
    assert_eq!(corrupt_err.disposition(), Disposition::Escalate);
    drop(corrupt_cur);
    drop(durable_a);
    close_durable_home(home_a, &request);
}

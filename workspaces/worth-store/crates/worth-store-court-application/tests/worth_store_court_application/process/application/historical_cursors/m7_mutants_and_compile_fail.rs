//! Store M4.4 `M7` production-path negative-mutant certification and
//! external-crate stable-rustc compile-fail fences verified directly against
//! real `worth_runtime_world` and `worth_store_application` compiled artifacts.

use std::path::PathBuf;
use std::process::Command;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    HistoricalBasisRetentionKind, HistoricalReadDenial, PROJECTION_CANONICAL_HEAD_ONLY,
};
use worth_store::pin_table::RootPinState;
use worth_store_application::{
    HistoricalChunkGeometry, HistoricalCursorCertificationMutant, MemoryHistoricalBasisStore,
};

use super::{
    close_durable_home, drain_cursor, make_rows, open_durable_home, test_budget, test_origin,
    test_query, test_query_with_projection, test_schema,
};

#[test]
fn m7_production_mutants_kill_s5_laws_on_directory_and_memory() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let schema = test_schema(1, 0xE1);
    let query = test_query(&schema);
    let query_head = test_query_with_projection(&schema, PROJECTION_CANONICAL_HEAD_ONLY);
    let origin = test_origin("main", 0xE2);
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 4,
        inline_history_limit: 2,
        continuation_page_versions: 2,
        layered_initial: true,
    };

    let (c1, c2, c3) = (CommitId(901), CommitId(999), CommitId(903));
    let c1_rows = make_rows(16, "m7-prod-c1", 4, 32);
    let c2_rows = make_rows(16, "m7-prod-head-c2", 4, 32);

    // 1. Directory backend production mutants (ALL 6 M7 mutants)
    let dir = tempfile::tempdir().unwrap();
    let home = open_durable_home(dir.path(), &request);
    let durable = home.historical_basis_store(budget).unwrap();
    let (c1_comm, _) = durable
        .retain_basis(
            c1,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            c1_rows.clone(),
            geom,
        )
        .unwrap();
    durable
        .retain_basis(
            c2,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::LiveBranch,
            c2_rows.clone(),
            geom,
        )
        .unwrap();
    let (c3_comm, _) = durable
        .retain_basis(
            c3,
            origin.clone(),
            schema.clone(),
            HistoricalBasisRetentionKind::PinLease,
            c1_rows.clone(),
            geom,
        )
        .unwrap();

    let mut cur = durable
        .open_historical_cursor(&execution, c1, c1_comm, query.clone(), &schema)
        .unwrap();
    let c1_wire = cur
        .next_page(&execution, 4)
        .unwrap()
        .checkpoint_token()
        .encode_wire();
    drop(cur);

    // Mutant A: ResumeByPhysicalOffset skips rows & diverges from canonical C1 tail after index discard!
    durable.discard_auxiliary_indexes();
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::ResumeByPhysicalOffset);
    let mut bad_offset_cur = durable
        .resume_cursor_wire(&execution, &c1_wire, query.clone(), &schema)
        .unwrap();
    let bad_tail = drain_cursor(&mut bad_offset_cur, &execution, 4);
    assert_ne!(
        bad_tail,
        c1_rows[4..],
        "ResumeByPhysicalOffset MUST violate canonical tail equality"
    );
    drop(bad_offset_cur);

    // Mutant B: SkipTokenMacVerification admits a tampered-MAC token that lawful code refuses!
    let mut tampered_bytes = c1_wire.clone().into_bytes();
    let idx = tampered_bytes.len() - 2;
    tampered_bytes[idx] = if tampered_bytes[idx] == b'A' {
        b'B'
    } else {
        b'A'
    };
    let tampered_wire = String::from_utf8(tampered_bytes).unwrap();
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::None);
    assert!(durable
        .resume_cursor_wire(&execution, &tampered_wire, query.clone(), &schema)
        .is_err());
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::SkipTokenMacVerification);
    let mutant_mac = durable.resume_cursor_wire(&execution, &tampered_wire, query.clone(), &schema);
    assert!(
        mutant_mac.is_ok(),
        "SkipTokenMacVerification MUST expose forged token admission"
    );
    drop(mutant_mac);

    // Mutant C: SkipQueryDigestValidation admits a Full-query token under HeadOnly!
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::None);
    assert!(matches!(
        durable
            .resume_cursor_wire(&execution, &c1_wire, query_head.clone(), &schema)
            .err(),
        Some(HistoricalReadDenial::UnsupportedHistoricalMeaning { .. })
    ));
    durable
        .set_certification_mutant(HistoricalCursorCertificationMutant::SkipQueryDigestValidation);
    let mutant_query =
        durable.resume_cursor_wire(&execution, &c1_wire, query_head.clone(), &schema);
    assert!(
        mutant_query.is_ok(),
        "SkipQueryDigestValidation MUST expose mismatched-query admission"
    );
    drop(mutant_query);

    // Mutant D: LocatorCoupledCommitment mutates commitment on consolidate & breaks pre-consolidation resume!
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::None);
    let mut c3_cur = durable
        .open_historical_cursor(&execution, c3, c3_comm, query.clone(), &schema)
        .unwrap();
    let c3_wire = c3_cur
        .next_page(&execution, 4)
        .unwrap()
        .checkpoint_token()
        .encode_wire();
    drop(c3_cur);
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::LocatorCoupledCommitment);
    let (mutated_comm, _) = durable.consolidate_basis(c3).unwrap();
    assert_ne!(
        mutated_comm, c3_comm,
        "LocatorCoupledCommitment MUST break commitment invariance"
    );
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::None);
    assert!(
        matches!(
            durable
                .resume_cursor_wire(&execution, &c3_wire, query.clone(), &schema)
                .err(),
            Some(HistoricalReadDenial::CorruptStorage { .. })
        ),
        "locator-coupled consolidation MUST break pre-consolidation token resume"
    );

    // Mutant E: SkipPinLoanAcquisition fails to hold pin in Expiring when pruned concurrently!
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::SkipPinLoanAcquisition);
    let unpinned_cur = durable
        .resume_cursor_wire(&execution, &c1_wire, query.clone(), &schema)
        .unwrap();
    assert_eq!(
        durable.prune_basis(c1).unwrap(),
        Some(RootPinState::Expired)
    );
    drop(unpinned_cur);

    // Mutant F: RebindToHeadOnPruned serves HEAD C2 rows instead of HistoryUnavailable for pruned C1!
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::None);
    assert_eq!(
        durable
            .resume_cursor_wire(&execution, &c1_wire, query.clone(), &schema)
            .err(),
        Some(HistoricalReadDenial::HistoryUnavailable { commit_id: c1 })
    );
    durable.set_certification_mutant(HistoricalCursorCertificationMutant::RebindToHeadOnPruned);
    let mut head_cur = durable
        .resume_cursor_wire(&execution, &c1_wire, query.clone(), &schema)
        .unwrap();
    assert_eq!(drain_cursor(&mut head_cur, &execution, 4), c2_rows[4..]);
    drop(head_cur);
    drop(durable);
    close_durable_home(home, &request);

    // 2. Memory parity: verify SkipQueryDigestValidation + RebindToHeadOnPruned!
    let mem = MemoryHistoricalBasisStore::new(
        HomeIdentity::from_observed_bytes([0xE3; 16]).unwrap(),
        HomeIncarnation::initial(),
        budget,
    );
    let pin_k = HistoricalBasisRetentionKind::PinLease;
    let live_k = HistoricalBasisRetentionKind::LiveBranch;
    let (m_c1_comm, _) = mem
        .retain_basis(c1, origin.clone(), schema.clone(), pin_k, c1_rows, geom)
        .unwrap();
    mem.retain_basis(c2, origin, schema.clone(), live_k, c2_rows.clone(), geom)
        .unwrap();
    let mut m_cur = mem
        .open_historical_cursor(&execution, c1, m_c1_comm, query.clone(), &schema)
        .unwrap();
    let m_wire = m_cur
        .next_page(&execution, 4)
        .unwrap()
        .checkpoint_token()
        .encode_wire();
    drop(m_cur);
    mem.set_certification_mutant(HistoricalCursorCertificationMutant::SkipQueryDigestValidation);
    assert!(mem
        .resume_cursor_wire(&execution, &m_wire, query_head, &schema)
        .is_ok());
    mem.set_certification_mutant(HistoricalCursorCertificationMutant::None);
    mem.prune_basis(c1).unwrap();
    mem.set_certification_mutant(HistoricalCursorCertificationMutant::RebindToHeadOnPruned);
    let mut m_bad = mem
        .resume_cursor_wire(&execution, &m_wire, query, &schema)
        .unwrap();
    assert_eq!(drain_cursor(&mut m_bad, &execution, 4), c2_rows[4..]);
}

fn find_rlib(deps_dir: &std::path::Path, crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut rlibs: Vec<PathBuf> = std::fs::read_dir(deps_dir)
        .expect("read deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".rlib"))
        })
        .collect();
    rlibs.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    rlibs
        .pop()
        .unwrap_or_else(|| panic!("compiled {prefix}*.rlib"))
}

fn assert_external_compile_fail(name: &str, source: &str, expected_fragments: &[&str]) {
    let exe = std::env::current_exe().expect("test executable path");
    let deps_dir = exe.parent().expect("deps directory").to_path_buf();
    let world_rlib = find_rlib(&deps_dir, "worth_runtime_world");
    let store_rlib = find_rlib(&deps_dir, "worth_store_application");
    let tmp = tempfile::tempdir().expect("tempdir for compile-fail probe");
    let src_path = tmp.path().join(format!("{name}.rs"));
    std::fs::write(&src_path, source).expect("write compile-fail probe source");
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let output = Command::new("rustc")
        .env("CARGO_PKG_NAME", "rogue-external-crate")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--emit=metadata",
            "--out-dir",
        ])
        .arg(&out_dir)
        .args(["-L", &format!("dependency={}", deps_dir.display())])
        .args([
            "--extern",
            &format!("worth_runtime_world={}", world_rlib.display()),
        ])
        .args([
            "--extern",
            &format!("worth_store_application={}", store_rlib.display()),
        ])
        .arg(&src_path)
        .output()
        .expect("invoke rustc for compile-fail fence");

    assert!(
        !output.status.success(),
        "compile-fail fence `{name}` unexpectedly compiled!"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    for fragment in expected_fragments {
        assert!(
            stderr.contains(fragment),
            "compile-fail `{name}` expected `{fragment}`, got:\n{stderr}"
        );
    }
}

#[test]
fn external_crate_compile_fail_fences_seal_tokens_loans_cursors_authority_and_projected_admission()
{
    // 1. BorrowedHistoricalRow cannot outlive HistoricalCursor
    assert_external_compile_fail(
        "real_borrowed_row_cannot_outlive_cursor",
        r#"
use worth_runtime_world::facade::HistoricalBasisRow;
use worth_store_application::{BorrowedHistoricalRow, HistoricalCursor};

pub fn escape<'a>(
    cursor: HistoricalCursor<'a, HistoricalBasisRow>,
) -> Option<BorrowedHistoricalRow<'a, HistoricalBasisRow>> {
    let borrowed = cursor.peek_borrowed_row();
    drop(cursor);
    borrowed
}
"#,
        &["E0505", "E0515"],
    );

    // 2. RetainedBasisToken private fields cannot be forged or mutated
    assert_external_compile_fail(
        "real_retained_basis_token_private_fields",
        r#"
use worth_store_application::RetainedBasisToken;

pub fn forge() -> RetainedBasisToken {
    RetainedBasisToken { format_version: 1 }
}

pub fn read_private(token: &RetainedBasisToken) -> u16 {
    token.format_version
}
"#,
        &["due to private fields", "E0616"],
    );

    // 3. Token forgery / arbitrary advancement constructors are unreachable outside worth-store-application
    assert_external_compile_fail(
        "real_token_forgery_methods_sealed",
        r#"
use worth_runtime_world::facade::{CanonicalCursorKey, RetainedBasisTokenFields};
use worth_store_application::RetainedBasisToken;

pub fn try_issue(fields: RetainedBasisTokenFields) {
    let _ = RetainedBasisToken::issue_with_key(fields, &[0u8; 32]);
}

pub fn try_advance(tok: &RetainedBasisToken) {
    let _ = tok.advance_with_key(CanonicalCursorKey::start(), &[0u8; 32]);
}
"#,
        &["E0624"],
    );

    // 4. External crate cannot implement sealed ReaderLoanGuard or construct ReaderLoan / HistoricalCursor
    assert_external_compile_fail(
        "real_loan_guard_and_cursor_sealed",
        r#"
use worth_store_application::{HistoricalCursor, ReaderLoan, ReaderLoanGuard};
pub struct FakeGuard;
impl ReaderLoanGuard for FakeGuard { fn is_active(&self) -> bool { true } }
pub fn fake_loan() {
    let _ = ReaderLoan::from_guard(Box::new(FakeGuard));
    let _ = HistoricalCursor::<u8>::from_owner;
}
"#,
        &["E0277", "E0624"],
    );

    // 5. Bypass Route 1: authority minting is completely absent & sealed!
    assert_external_compile_fail(
        "bypass_route_1_const_bool_and_store_authority_sealed",
        r#"
pub fn forge_with_const_bool() {
    let _ = worth_runtime_world::__historical_cursor_seal::HistoricalCursorAuthorityWitness::__from_macro_checked::<true>();
    let _ = worth_runtime_world::admit_historical_cursor_authority!([0u8; 32]);
    let _ = worth_store_application::HistoricalCursorAuthority::new([0u8; 32]);
}
"#,
        &["E0433"],
    );

    // 6. Bypass Route 2: custom page producer, open_or_restore, and unsealed from_progress are sealed!
    assert_external_compile_fail(
        "bypass_route_2_custom_page_producer_and_open_or_restore_sealed",
        r#"
use worth_store_application::{AdmittedQueryWindowPage, DurableHistoricalBasisStore, MemoryHistoricalBasisStore};
pub fn try_custom_producer(store: &DurableHistoricalBasisStore, mem: &MemoryHistoricalBasisStore) {
    let _ = store.open_projected_cursor();
    let _ = mem.open_projected_cursor();
    let _ = DurableHistoricalBasisStore::open_or_restore;
    let _ = AdmittedQueryWindowPage::<u8>::from_progress;
}
pub fn try_forge_page(page: AdmittedQueryWindowPage<u8>) {
    let _ = page.inner;
}
"#,
        &["E0599", "E0624", "E0616", "from_progress"],
    );
}

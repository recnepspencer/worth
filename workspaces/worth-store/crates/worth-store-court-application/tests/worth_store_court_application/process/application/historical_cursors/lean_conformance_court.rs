//! Executable Rust↔Lean conformance court verifying real Directory Home traces
//! (payload/history digests, cursor positions, actual Store pin/basis counts, R1/C1/C2/P1
//! crash cuts) plus 3 negative controls (HEAD rebind, payload mutation, pin-count mismatch).

use std::path::PathBuf;
use std::process::Command;

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    HistoricalBasisRetentionKind, HistoricalBasisRow, HistoricalReadDenial,
};
use worth_store_application::{DurableHistoricalCrashCut, HistoricalChunkGeometry};

use super::{
    close_durable_home, drain_cursor, make_rows, open_durable_home, test_budget, test_origin,
    test_query, test_schema,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("repo root 4 levels above worth-store-court-application")
        .to_path_buf()
}

fn run_lean_conformance(trace_path: &std::path::Path) -> std::process::Output {
    let lean_dir = repo_root().join("proofs/lean/store_m4");
    let exe = lean_dir.join(".lake/build/bin").join(if cfg!(windows) {
        "retained_basis_conformance.exe"
    } else {
        "retained_basis_conformance"
    });
    if exe.exists() {
        Command::new(&exe)
            .arg(trace_path)
            .output()
            .expect("invoke retained_basis_conformance")
    } else {
        Command::new("lake")
            .arg("--dir")
            .arg(&lean_dir)
            .args(["exe", "retained_basis_conformance"])
            .arg(trace_path)
            .output()
            .expect("invoke lake exe")
    }
}

fn digest_u64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

fn row_key_num(row: &HistoricalBasisRow) -> usize {
    std::str::from_utf8(&row.logical_key)
        .unwrap()
        .strip_prefix("key-")
        .unwrap()
        .parse()
        .unwrap()
}

fn encode_row(row: &HistoricalBasisRow) -> String {
    let hs = if row.history_versions.is_empty() {
        "-".to_owned()
    } else {
        row.history_versions
            .iter()
            .map(|h| digest_u64(h).to_string())
            .collect::<Vec<_>>()
            .join(";")
    };
    format!("{}:{}:{hs}", row_key_num(row), digest_u64(&row.payload))
}

fn encode_rows(rows: &[HistoricalBasisRow]) -> String {
    if rows.is_empty() {
        "-".to_owned()
    } else {
        rows.iter().map(encode_row).collect::<Vec<_>>().join(",")
    }
}

fn outcome_label(
    res: Result<(Option<usize>, Vec<HistoricalBasisRow>), HistoricalReadDenial>,
    commit_id: u64,
) -> String {
    match res {
        Ok((pos, rows)) => format!(
            "admitted:{commit_id}:{}:{}",
            pos.map_or_else(|| "-".to_owned(), |k| k.to_string()),
            encode_rows(&rows)
        ),
        Err(HistoricalReadDenial::HistoryUnavailable { .. }) => "historyUnavailable".to_owned(),
        Err(HistoricalReadDenial::UnsupportedHistoricalMeaning { .. }) => {
            "unsupportedMeaning".to_owned()
        }
        Err(HistoricalReadDenial::CorruptStorage { .. }) => "corruptStorage".to_owned(),
        Err(other) => panic!("unexpected denial: {other:?}"),
    }
}

fn push_pins(
    trace: &mut Vec<String>,
    store: &worth_store_application::DurableHistoricalBasisStore,
) {
    trace.push(format!(
        "ASSERT_PINS {} {} {}",
        store.live_pin_count(),
        store.pending_retire_pin_count(),
        store.retained_basis_count()
    ));
}

fn resume_and_drain(
    store: &worth_store_application::DurableHistoricalBasisStore,
    exec: &ExecutionRequest<'_, '_>,
    wire: &str,
    query: worth_runtime_world::facade::HistoricalQueryContract,
    schema: &worth_runtime_world::facade::SchemaVersionIdentity,
) -> Result<(Option<usize>, Vec<HistoricalBasisRow>), HistoricalReadDenial> {
    store
        .resume_cursor_wire(exec, wire, query, schema)
        .map(|mut cur| {
            let rows = drain_cursor(&mut cur, exec, 2);
            (rows.last().map(row_key_num), rows)
        })
}

fn assert_mutant_rejected(tmp: &std::path::Path, name: &str, lines: &[String], msg: &str) {
    let path = tmp.join(name);
    std::fs::write(&path, lines.join("\n")).unwrap();
    let out = run_lean_conformance(&path);
    assert!(
        !out.status.success()
            && String::from_utf8_lossy(&out.stderr).contains("CONFORMANCE FAILED"),
        "{msg}"
    );
}

#[test]
fn lean_conformance_verifies_real_rust_trace_payloads_history_pins_and_rejects_mutations() {
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let budget = test_budget(128 * 1024, 64 * 1024);
    let (schema_v1, schema_v2) = (test_schema(1, 0xF1), test_schema(2, 0xF2));
    let (query, origin) = (test_query(&schema_v1), test_origin("main", 0xF3));
    let geom = HistoricalChunkGeometry {
        rows_per_chunk: 2,
        inline_history_limit: 1,
        continuation_page_versions: 1,
        layered_initial: true,
    };
    let (c1, c2, c_cut) = (CommitId(101), CommitId(102), CommitId(103));
    let c1_rows = make_rows(6, "lean-c1", 2, 24);
    let c2_rows = make_rows(6, "lean-c2", 2, 24);
    let cut_rows = make_rows(4, "lean-cut-r1", 2, 24);
    let mut trace = vec!["# trace".to_owned(), "INIT 1 77 1 1 42".to_owned()];
    let (dir_a, dir_b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

    let cross_home_wire = {
        let hb = open_durable_home(dir_b.path(), &request);
        let sb = hb.historical_basis_store(budget).unwrap();
        let (cb, _) = sb
            .retain_basis(
                c1,
                origin.clone(),
                schema_v1.clone(),
                HistoricalBasisRetentionKind::PinLease,
                c1_rows.clone(),
                geom,
            )
            .unwrap();
        let mut cur = sb
            .open_historical_cursor(&execution, c1, cb, query.clone(), &schema_v1)
            .unwrap();
        let wire = cur
            .next_page(&execution, 2)
            .unwrap()
            .checkpoint_token()
            .encode_wire();
        drop(cur);
        drop(sb);
        close_durable_home(hb, &request);
        wire
    };

    // Phase 1 on Home A: retain C1, page 1 (keys 0,1), advance C2, C1/C2 cuts + consolidate C1,
    // R1 crash cut on C3, close & reopen.
    let c1_wire = {
        let home = open_durable_home(dir_a.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        let (c1_comm, _) = store
            .retain_basis(
                c1,
                origin.clone(),
                schema_v1.clone(),
                HistoricalBasisRetentionKind::PinLease,
                c1_rows.clone(),
                geom,
            )
            .unwrap();
        trace.push(format!("RETAIN 101 1 500 {}", encode_rows(&c1_rows)));
        push_pins(&mut trace, &store);

        let mut cur = store
            .open_historical_cursor(&execution, c1, c1_comm, query.clone(), &schema_v1)
            .unwrap();
        let p1 = cur.next_page(&execution, 2).unwrap();
        assert_eq!(
            p1.rows().iter().map(row_key_num).collect::<Vec<_>>(),
            [0, 1]
        );
        let wire = p1.checkpoint_token().encode_wire();
        drop(cur);

        store
            .retain_basis(
                c2,
                origin.clone(),
                schema_v1.clone(),
                HistoricalBasisRetentionKind::LiveBranch,
                c2_rows.clone(),
                geom,
            )
            .unwrap();
        trace.push(format!("RETAIN 102 1 502 {}", encode_rows(&c2_rows)));
        trace.push(format!("ADVANCE_HEAD 102 {}", encode_rows(&c2_rows)));

        store.arm_crash_cut(Some(
            DurableHistoricalCrashCut::AfterConsolidateNewPinBeforeBind,
        ));
        assert_eq!(
            store.consolidate_basis(c1).unwrap_err(),
            HistoricalReadDenial::RequestStopped
        );
        trace.push("CUT_C1_STAGE_CONSOLIDATE 101".to_owned());
        trace.push("CUT_C2_PIN_CONSOLIDATE 101".to_owned());

        drop(store);
        close_durable_home(home, &request);
        trace.push("REOPEN".to_owned());
        wire
    };

    // Phase 2 on reopened Home A: verify pin counts, consolidation, payload/history digests,
    // aux index discard, cross-Home forgery, wrong schema, and R1/P1 crash cuts!
    {
        let home = open_durable_home(dir_a.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        push_pins(&mut trace, &store);

        store.consolidate_basis(c1).unwrap();
        trace.push("CONSOLIDATE 101".to_owned());
        push_pins(&mut trace, &store);

        let res_ok = resume_and_drain(&store, &execution, &c1_wire, query.clone(), &schema_v1);
        trace.push(format!(
            "RESUME 77 1 1 1 101 500 1 42 1 {}",
            outcome_label(res_ok, 101)
        ));
        trace.push("RELEASE 101".to_owned());

        store.discard_auxiliary_indexes();
        let after_discard = store
            .resume_cursor_wire(&execution, &c1_wire, query.clone(), &schema_v1)
            .map(|mut cur| drain_cursor(&mut cur, &execution, 2))
            .unwrap();
        let chunks_encoded = format!(
            "{}|{}|{}",
            encode_rows(&c1_rows[0..2]),
            encode_rows(&c1_rows[2..4]),
            encode_rows(&c1_rows[4..6]),
        );
        trace.push(format!(
            "ASSERT_CHUNK_INDEX {chunks_encoded} 1 {}",
            encode_rows(&after_discard)
        ));

        let res_forged = resume_and_drain(
            &store,
            &execution,
            &cross_home_wire,
            query.clone(),
            &schema_v1,
        );
        trace.push(format!(
            "RESUME 99 2 1 1 101 500 1 42 1 {}",
            outcome_label(res_forged, 101)
        ));

        let res_schema = resume_and_drain(&store, &execution, &c1_wire, query.clone(), &schema_v2);
        trace.push(format!(
            "RESUME 77 1 1 1 101 500 2 42 1 {}",
            outcome_label(res_schema, 101)
        ));

        store.arm_crash_cut(Some(DurableHistoricalCrashCut::AfterRetainStageBeforePin));
        assert_eq!(
            store
                .retain_basis(
                    c_cut,
                    origin,
                    schema_v1.clone(),
                    HistoricalBasisRetentionKind::PinLease,
                    cut_rows.clone(),
                    geom
                )
                .unwrap_err(),
            HistoricalReadDenial::RequestStopped
        );
        trace.push(format!("CUT_R1_STAGE 103 1 503 {}", encode_rows(&cut_rows)));

        store.arm_crash_cut(Some(
            DurableHistoricalCrashCut::AfterPruneCatalogBeforePinExpire,
        ));
        assert_eq!(
            store.prune_basis(c1).unwrap_err(),
            HistoricalReadDenial::RequestStopped
        );
        trace.push("CUT_P1_PRUNE_CATALOG 101".to_owned());
        drop(store);
        close_durable_home(home, &request);
        trace.push("REOPEN".to_owned());
    }

    // Phase 3 on reopened Home A: verify exact pin counts (only C2 live) and pruned C1 refuses!
    {
        let home = open_durable_home(dir_a.path(), &request);
        let store = home.historical_basis_store(budget).unwrap();
        push_pins(&mut trace, &store);
        let res_pruned = resume_and_drain(&store, &execution, &c1_wire, query, &schema_v1);
        trace.push(format!(
            "RESUME 77 1 1 1 101 500 1 42 1 {}",
            outcome_label(res_pruned, 101)
        ));
        drop(store);
        close_durable_home(home, &request);
    }

    let tmp = tempfile::tempdir().unwrap();
    let valid_path = tmp.path().join("valid.trace");
    std::fs::write(&valid_path, trace.join("\n")).unwrap();
    let ok_out = run_lean_conformance(&valid_path);
    assert!(
        ok_out.status.success()
            && String::from_utf8_lossy(&ok_out.stdout).contains("CONFORMANCE OK"),
        "Lean conformance failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ok_out.stdout),
        String::from_utf8_lossy(&ok_out.stderr),
    );

    let mut head_mutant = trace.clone();
    let last = head_mutant.last_mut().unwrap();
    *last = last.replace(
        "historyUnavailable",
        &format!("admitted:102:5:{}", encode_rows(&c2_rows[2..])),
    );
    assert_mutant_rejected(
        tmp.path(),
        "head_mutant.trace",
        &head_mutant,
        "Lean MUST reject HEAD-rebind",
    );

    let mut payload_mutant = trace.clone();
    let target_line = payload_mutant
        .iter_mut()
        .find(|line| line.starts_with("RESUME 77 1 1 1 101 500 1 42 1 admitted:101:"))
        .expect("admitted RESUME line");
    let orig = digest_u64(&c1_rows[2].payload);
    *target_line = target_line.replacen(&orig.to_string(), &orig.wrapping_add(1).to_string(), 1);
    assert_mutant_rejected(
        tmp.path(),
        "payload_mutant.trace",
        &payload_mutant,
        "Lean MUST reject mutated payload",
    );

    let mut pin_mutant = trace;
    *pin_mutant
        .iter_mut()
        .rev()
        .find(|l| l.starts_with("ASSERT_PINS "))
        .expect("ASSERT_PINS") = "ASSERT_PINS 99 0 1".to_owned();
    assert_mutant_rejected(
        tmp.path(),
        "pin_mutant.trace",
        &pin_mutant,
        "Lean MUST reject mismatched pin count",
    );
}

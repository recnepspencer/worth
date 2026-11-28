//! External-crate stable-rustc compile-fail fences proving:
//! 1. No external crate can import `store_cursor_boundary`, construct a sealing
//!    authority with a caller-chosen key (`from_persisted_home_key`), call
//!    `seal_canonical_bytes`/`open_canonical_bytes`, extract `last_boundary`/
//!    `after_boundary`, or read private ordering values.
//! 2. No external crate can name Store's crate-private `HistoricalCursorAuthority`.
//! 3. No external crate can plug an un-accounted handwritten row (missing
//!    `KnownProjectedHeapBytes`) into `AdmittedQueryWindowSource`.

use std::path::PathBuf;
use std::process::Command;

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

fn run_external_probe(
    deps_dir: &std::path::Path,
    exec_rlib: &std::path::Path,
    store_rlib: &std::path::Path,
    name: &str,
    source: &str,
) -> String {
    let tmp = tempfile::tempdir().expect("tempdir for compile-fail probe");
    let src_path = tmp.path().join(format!("{name}.rs"));
    let out_dir = tmp.path().join("out");
    std::fs::create_dir_all(&out_dir).expect("create out dir");
    std::fs::write(&src_path, source).expect("write compile-fail probe");

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
            &format!("worth_query_execution={}", exec_rlib.display()),
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
        "external probe `{name}` unexpectedly compiled!"
    );
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn external_crate_cannot_seal_window_boundary_with_chosen_key_or_extract_withheld_order_values() {
    let exe = std::env::current_exe().expect("test executable path");
    let deps_dir = exe.parent().expect("deps directory").to_path_buf();
    let exec_rlib = find_rlib(&deps_dir, "worth_query_execution");
    let store_rlib = find_rlib(&deps_dir, "worth_store_application");

    let stderr = run_external_probe(
        &deps_dir,
        &exec_rlib,
        &store_rlib,
        "boundary_key_forgery",
        r#"
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationRootWindow,
    WorthQueryApplicationRootWindowBoundary,
    WorthQueryApplicationRootWindowPage,
    WorthQueryApplicationRootWindowProgress,
};

pub fn try_inspect_window_after(window: &WorthQueryApplicationRootWindow) {
    let _ = &window.after;
    let _ = window.after_position();
}

pub fn try_import_removed_boundary_sealing_module() {
    let _ = worth_query_execution::store_cursor_boundary::HomeWindowBoundarySealingAuthority::from_persisted_home_key([0x42u8; 32]);
}

pub fn try_access_store_private_authority() {
    let _ = worth_store_application::HistoricalCursorAuthority::from_home_key([0x42u8; 32]);
}

pub fn try_seal_with_attacker_key(boundary: &WorthQueryApplicationRootWindowBoundary) {
    let _ = boundary.seal_canonical_bytes(&[0x42u8; 32], b"attacker-aad");
}

pub fn try_open_with_attacker_key(bytes: &[u8]) {
    let _ = WorthQueryApplicationRootWindowBoundary::open_canonical_bytes(
        bytes,
        &[0x42u8; 32],
        b"attacker-aad",
    );
}

pub fn try_read_raw_ordering(boundary: &WorthQueryApplicationRootWindowBoundary) {
    let _ = &boundary.ordering;
}

pub fn try_extract_boundaries_and_anchor(
    page: &WorthQueryApplicationRootWindowPage<u8>,
    progress: &WorthQueryApplicationRootWindowProgress,
) {
    let _ = page.progress().last_boundary();
    let _ = progress.after_boundary();
    let _ = progress.after_root_entity();
    let _ = progress.resume_origin();
}

pub fn try_call_removed_branch_reconcile<S>(
    branches: &worth_query_execution::facade::primary_graph::WorthQueryApplicationProductBranches<'_, S>,
) {
    let _ = branches.reconcile_orphaned_branch_on_open();
}
"#,
    );
    for expected in [
        "store_cursor_boundary",
        "HistoricalCursorAuthority",
        "seal_canonical_bytes",
        "open_canonical_bytes",
        "last_boundary",
        "after_boundary",
        "after_root_entity",
        "resume_origin",
        "reconcile_orphaned_branch_on_open",
        "after_position",
        "E0433",
        "E0599",
        "E0616",
    ] {
        assert!(
            stderr.contains(expected),
            "expected `{expected}` in rustc stderr, got:\n{stderr}"
        );
    }

    let window_dbg = format!(
        "{:?}",
        worth_query_execution::facade::primary_graph::WorthQueryApplicationRootWindow::after_root_entity_bound(
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId(7),
                12345,
                99,
            ),
            [0xAB; 32],
            std::num::NonZeroUsize::new(4).unwrap(),
        )
    );
    assert!(
        !window_dbg.contains("12345")
            && !window_dbg.contains("99")
            && !window_dbg.contains("171")
            && !window_dbg.contains("ab"),
        "WorthQueryApplicationRootWindow Debug must redact anchor EntityId and nonce, got: {window_dbg}"
    );

    let stderr_row = run_external_probe(
        &deps_dir,
        &exec_rlib,
        &store_rlib,
        "unaccounted_row_rejected",
        r#"
pub struct UnaccountedHandwrittenRow {
    pub hidden_buf: String,
}

pub fn require_known_heap_bytes<S: worth_store_application::AdmittedQueryWindowSource<UnaccountedHandwrittenRow>>(_: S) {}
"#,
    );
    assert!(
        stderr_row.contains("KnownProjectedHeapBytes") && stderr_row.contains("E0277"),
        "un-accounted handwritten row must fail to satisfy KnownProjectedHeapBytes bound, got:\n{stderr_row}"
    );
}

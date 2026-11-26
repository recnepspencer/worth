//! W8C-1: an application rule's structured refusal (`code` + `message`)
//! reaches the caller unchanged from a hand-written handler and from a
//! commit-boundary invariant, across sync and background session calls; the
//! Home is untouched; authority and session refusals never carry one; and an
//! inadmissible reason is a deterministic handler defect, never truncated.
mod authority_court;
mod handler;
mod handler_court;
mod invariant;
mod invariant_court;
mod program;
mod schema;

use std::path::PathBuf;

use program::{principal, StockHome};
use schema::{LotView, ReadLot};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;
use worth_query_host::facade::{WorthQueryDomainRefusal, WorthQuerySessionRunDenial};

type RunDenial = WorthQuerySessionRunDenial<WorthQueryOperationBodyDenial>;

fn temp_home_path(label: &str) -> PathBuf {
    let mut path =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("domain-refusal-{label}"));
    path.set_extension(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&path);
    path
}

/// The lot as committed, read back by the owner.
fn lot(home: &StockHome, clock: &WorthQueryAdmissionClock, lot: u64) -> LotView {
    let mut rows = home
        .session(principal(home, clock, "owner"))
        .query(ReadLot { lot })
        .expect("owner reads the lot");
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}

fn assert_refusal(refusal: Option<WorthQueryDomainRefusal>, code: &str, message: &str) {
    let refusal = refusal.expect("the run carries a domain refusal");
    assert_eq!((refusal.code(), refusal.message()), (code, message));
}

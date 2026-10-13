//! Prepared native marking diagnostics, never publication or currentness proof.
use std::cell::{Cell, RefCell};
use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::history::CommitId;

use super::super::RecordedSettlementIdentity;
use super::logical_marking::{NativeMarkingReport, WorthQueryNativeMarkingPrecision};
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::execution_observation;

/// One affected output described by its actual lineage identity. These copied
/// fields cannot select a settlement or authorize a graph operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMarkedOutputForTest {
    pub runtime_authority: u64,
    pub root: WorthQueryOperationScopeEntityBinding,
    pub output_binding: Arc<str>,
    pub product_incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    pub generation: u64,
    pub slot: usize,
}

/// Work and affected outputs from preparing one native delivery. A later
/// publication refusal may discard it. `None` means exact affected outputs
/// were unavailable, never that no output was affected.
#[derive(Clone, Debug)]
pub struct WorthQueryNativeMarkingForTest {
    pub commit: CommitId,
    pub precision: WorthQueryNativeMarkingPrecision,
    /// Existing marking-meter debit for matching, propagation and selected
    /// hint preparation. Navigation/checkpoints and touch extraction are not
    /// charged by that meter; this is not total physical publication work.
    /// `None` is an eviction fallback without a completed delivery report:
    /// its canonical precision carries default counts, not measured work.
    pub charged_work: Option<u64>,
    pub affected: Option<Vec<WorthQueryMarkedOutputForTest>>,
}

thread_local! {
    static REPORTS: RefCell<Vec<WorthQueryNativeMarkingForTest>> = const { RefCell::new(Vec::new()) };
    static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn record(
    report: &NativeMarkingReport,
    affected: Option<&OrdSet<Arc<RecordedSettlementIdentity>>>,
    charged_work: Option<u64>,
) {
    if !execution_observation::is_installed() {
        return;
    }
    if !CLEANUP_REGISTERED.replace(true) {
        execution_observation::register_cleanup(clear);
    }
    let affected = affected.map(|identities| {
        identities
            .iter()
            .map(|identity| {
                let source = identity.source();
                let (product_incarnation, generation, slot) = identity.address();
                WorthQueryMarkedOutputForTest {
                    runtime_authority: source.runtime_authority,
                    root: source.scope,
                    output_binding: Arc::from(source.output_binding.as_str()),
                    product_incarnation,
                    generation,
                    slot,
                }
            })
            .collect()
    });
    REPORTS.with_borrow_mut(|reports| {
        reports.push(WorthQueryNativeMarkingForTest {
            commit: report.commit,
            precision: report.precision,
            charged_work,
            affected,
        })
    });
}

/// Drain prepared native delivery reports while the existing owned Query
/// execution observation is live. This function grants no caller authority.
#[doc(hidden)]
pub fn native_marking_on_this_thread_for_test() -> Vec<WorthQueryNativeMarkingForTest> {
    REPORTS.with_borrow_mut(std::mem::take)
}

fn clear() {
    CLEANUP_REGISTERED.set(false);
    REPORTS.with_borrow_mut(Vec::clear);
}

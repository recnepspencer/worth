//! Fintech workflow actions.
//!
//! This module is intentionally a table of contents. Branching, correction,
//! risk, settlement, savepoint, recovery, merge, and audit actions are split
//! by domain concern.

mod audits;
mod branching;
mod corrections;
mod merges;
mod metadata;
mod recovery;
mod risk;
mod savepoints;
mod settlements;
mod snapshots;

pub(super) use audits::emit_trade_correction_audit_record;
pub(super) use branching::{open_analysis_branch, open_audit_branch};
pub(super) use corrections::correct_seeded_trade_candidate;
pub(super) use merges::{diverge_case_trade_on_branch, merge_branch_into_main};
pub(super) use metadata::{build_branch_scoped_case_index, register_case_book_index};
pub(super) use recovery::{checkpoint_world, recover_persisted_world};
pub(super) use risk::{refresh_risk_views, shock_market_on_branch, stress_seeded_intraday_risk};
pub(super) use savepoints::{
    commit_case_trade_after_savepoint, rollback_case_trade_after_savepoint,
};
pub(super) use settlements::repair_seeded_failed_settlement;
pub(super) use snapshots::{capture_world_snapshot, release_snapshot_handle};

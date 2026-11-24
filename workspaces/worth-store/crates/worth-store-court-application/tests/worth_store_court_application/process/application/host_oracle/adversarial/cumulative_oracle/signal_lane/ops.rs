//! Signal + Bridge multi-branch observation, fork, parity, barrier-kill, and exact receipt-binding helpers.
use super::super::super::super::graceful::signal::operations::signal_oracle_details;
use super::super::super::assert_barrier_killed;
use super::super::super::dimensions_signal::wire::{WireCustody, WireReceipt, WireSnapshot};
use super::super::super::signal_bridge_io_kills::{admitted_signal_key_identity, SignalHome};
use super::super::StepCtx;
use super::child::{BARRIER_CHILD, ENV_BSPEC, ENV_BUILD, ENV_CH, ENV_HOME};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use worth_query_host::facade::product::{self, WorthQueryProductBranch};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::publication_limit;
use worth_store::physical_runtime::certification::MediaOperationRole;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct MultiBranchSnap {
    pub(super) branches: Vec<WireSnapshot>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum CumSignalCmd {
    ForkBranch,
    CommitMixed { branch_idx: usize, ordinal: u8 },
    PublishDef { branch_idx: usize },
    ContinueCustody,
    VerifyRecoveredBridge { branch_idx: usize },
    CheckpointAndDiscard,
    Close { expect_discarded: bool },
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub(super) struct CumSignalReply {
    pub(super) snap: Option<MultiBranchSnap>,
    pub(super) custody: Vec<WireCustody>,
    pub(super) receipt: Option<WireReceipt>,
    pub(super) initialized: usize,
    pub(super) contacts: (usize, usize, usize, usize),
    pub(super) cadence: BTreeMap<String, (u64, u64)>,
    pub(super) commit_tag: Option<String>,
    pub(super) definition_identity: Option<[u8; 32]>,
    pub(super) discard_gen: Option<u64>,
    pub(super) close_gen: Option<u64>,
    pub(super) closed: bool,
}

pub(super) fn parse_barrier_finish(note: &str) -> Option<(u64, &str)> {
    let rest = note.trim().strip_prefix("missed:actual=")?;
    let (actual_s, outcome) = rest.split_once(":outcome=")?;
    Some((actual_s.parse().ok()?, outcome))
}

pub(super) fn measure_cum_signal_barriers(
    home: &Path,
    branch_idx: usize,
    ordinal: u8,
    build: &str,
) -> u64 {
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("{branch_idx}|{ordinal}|{}", u64::MAX));
    let end = crate::killed_child::run_and_kill_with_ctx(
        BARRIER_CHILD,
        &[
            (ENV_HOME, home),
            (ENV_CH, markers.path()),
            (ENV_BUILD, Path::new(build)),
            (ENV_BSPEC, &spec),
        ],
        &markers.path().join("ready"),
        &markers.path().join("finished"),
        &format!("signal-calibration(branch={branch_idx},ord={ordinal},build={build})"),
    );
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    let note = std::fs::read_to_string(markers.path().join("finished")).unwrap();
    let (actual, outcome) = parse_barrier_finish(&note)
        .unwrap_or_else(|| panic!("signal barrier calibration malformed finish: {note:?}"));
    assert_eq!(
        outcome, "committed",
        "signal barrier calibration must commit (note={note:?})"
    );
    actual
}

pub(super) fn kill_cum_signal_barrier(
    home: &Path,
    branch_idx: usize,
    ordinal: u8,
    build: &str,
    nth: u64,
    ctx: &str,
) {
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("{branch_idx}|{ordinal}|{nth}"));
    let ready = markers.path().join("ready");
    let end = crate::killed_child::run_and_kill_with_ctx(
        BARRIER_CHILD,
        &[
            (ENV_HOME, home),
            (ENV_CH, markers.path()),
            (ENV_BUILD, Path::new(build)),
            (ENV_BSPEC, &spec),
        ],
        &ready,
        &markers.path().join("finished"),
        ctx,
    );
    assert_barrier_killed(
        end,
        &ready,
        MediaOperationRole::SynchronizeFileState,
        nth,
        ctx,
    );
}

pub(super) fn sorted_branches(home: &SignalHome) -> Vec<WorthQueryProductBranch> {
    let app = home.runtime();
    let inv = app
        .with_host_advancement(|phase| app.branches().inventory(&phase))
        .unwrap()
        .unwrap();
    let mut list: Vec<_> = inv.iter().collect();
    list.sort_by_key(|b| b.occurrence_ordinal());
    list
}

pub(super) fn fork_on_home(home: &SignalHome) {
    let app = home.runtime();
    let main = app.current_world();
    let ans = app
        .with_host_advancement(|phase| {
            app.branches()
                .fork(main)
                .components(|c| c.fork_relational().fork_signal())
                .create(&phase)
        })
        .unwrap();
    match ans {
        Ok(_) => {}
        Err(product::WorthQueryProductBranchCreateError::ProductUnpublished(mut orig)) => loop {
            match app
                .with_host_advancement(|phase| orig.continue_in_advancement(&phase))
                .unwrap()
            {
                Ok(product::WorthQueryProductBranchCreationContinuation::Performed(_)) => break,
                Err(f) => {
                    orig = f.into_recovery();
                    std::thread::yield_now();
                }
                other => panic!("signal fork continuation failed: {other:?}"),
            }
        },
        Err(e) => panic!("signal fork refused: {e:?}"),
    }
}

pub(super) fn observe_multi_branch(home: &SignalHome) -> MultiBranchSnap {
    let app = home.runtime();
    let branches = sorted_branches(home);
    let views = branches
        .into_iter()
        .map(|branch| {
            let row = publication_limit::read_intent_on_branch(app, branch);
            let details = signal_oracle_details(home, branch);
            let (world_commit_ordinal, branch_id, truth_version, schema_commitment, posture) = app
                .with_host_advancement(|phase| {
                    let selected = app.on_branch(branch).select(&phase).unwrap();
                    let prod = selected.product();
                    let desc = prod.relational_basis_descriptor();
                    (
                        prod.selected_commit().ordinal(),
                        format!("{:?}", desc.branch_id()),
                        desc.truth_version().as_u64(),
                        desc.schema_commitment(),
                        format!("{:?}", desc.posture()),
                    )
                })
                .unwrap();
            WireSnapshot {
                intent: (row.identity, row.revision, row.lifecycle, row.input),
                world_commit_ordinal,
                signal_generation: details.signal_generation,
                world_reference_generation: details.world_reference_generation,
                installed_conditional_definition_count: details
                    .installed_conditional_definition_count,
                installed_conditional_target_ref_count: details
                    .installed_conditional_target_ref_count,
                owned_signal_active_node_count: details.owned_signal_active_node_count,
                head_attempt_ordinal: details.head_attempt_ordinal,
                head_native_commit_id: details.head_native_commit_id,
                head_native_version_id: details.head_native_version_id,
                head_relational_changed: details.head_relational_changed,
                head_signal_changed: details.head_signal_changed,
                head_signal_movement: details.head_signal_movement,
                branch_id,
                truth_version,
                schema_commitment,
                posture,
            }
        })
        .collect();
    MultiBranchSnap { branches: views }
}

pub(super) fn assert_multi_parity(
    actual: &MultiBranchSnap,
    expected: &MultiBranchSnap,
    ctx: &StepCtx,
) {
    assert_eq!(
        actual.branches.len(),
        expected.branches.len(),
        "{ctx}: Signal branch count mismatch"
    );
    let n = actual.branches.len();
    for (idx, (a, e)) in actual.branches.iter().zip(&expected.branches).enumerate() {
        assert_eq!(a.intent, e.intent, "{ctx}: branch#{idx} intent");
        assert_eq!(
            a.signal_generation, e.signal_generation,
            "{ctx}: branch#{idx} signal_generation"
        );
        assert_eq!(
            a.world_reference_generation, e.world_reference_generation,
            "{ctx}: branch#{idx} world_reference_generation"
        );
        assert!(
            a.installed_conditional_definition_count >= n,
            "{ctx}: branch#{idx} def count {} < branches {n}",
            a.installed_conditional_definition_count
        );
        assert_eq!(
            a.installed_conditional_target_ref_count, a.installed_conditional_definition_count,
            "{ctx}: branch#{idx} target ref count"
        );
        assert_eq!(
            a.owned_signal_active_node_count,
            e.owned_signal_active_node_count * n,
            "{ctx}: branch#{idx} owned signal active nodes"
        );
        assert_eq!(
            (
                a.head_relational_changed,
                a.head_signal_changed,
                &a.head_signal_movement,
                &a.branch_id,
                a.truth_version,
                a.schema_commitment,
                &a.posture
            ),
            (
                e.head_relational_changed,
                e.head_signal_changed,
                &e.head_signal_movement,
                &e.branch_id,
                e.truth_version,
                e.schema_commitment,
                &e.posture
            ),
            "{ctx}: branch#{idx} head/basis parity mismatch"
        );
    }
}

/// Strictly bind a live or recovered Signal receipt to the exact durable World
/// occurrence & Native commit coordinates — never optional.
pub(super) fn assert_receipt_bound(
    receipt: &WireReceipt,
    expected_ordinal: u8,
    branch_snap: &WireSnapshot,
    ctx: &StepCtx,
) {
    assert_eq!(
        receipt.idempotency_key,
        admitted_signal_key_identity(expected_ordinal),
        "{ctx}: receipt idempotency key must match admitted key"
    );
    assert_eq!(
        receipt.composite_commit, branch_snap.world_commit_ordinal,
        "{ctx}: receipt world commit must match observed branch head"
    );
    assert_eq!(
        receipt.product_generation, branch_snap.world_reference_generation,
        "{ctx}: receipt product generation must match observed branch head"
    );
    assert_eq!(
        Some(receipt.attempt_ordinal),
        branch_snap.head_attempt_ordinal,
        "{ctx}: receipt attempt ordinal must match exact durable World occurrence"
    );
    assert_eq!(
        Some(receipt.commit_id),
        branch_snap.head_native_commit_id,
        "{ctx}: receipt native commit id must match exact durable World occurrence"
    );
    assert_eq!(
        Some(receipt.version_id),
        branch_snap.head_native_version_id,
        "{ctx}: receipt native version id must match exact durable World occurrence"
    );
    assert_eq!(
        receipt.truth_version, branch_snap.truth_version,
        "{ctx}: receipt truth version must match observed branch head"
    );
}

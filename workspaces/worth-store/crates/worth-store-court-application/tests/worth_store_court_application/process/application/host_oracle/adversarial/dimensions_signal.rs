//! C4b.3: Orthogonal build-identity change (Same / Changed / Alternating) and
//! D14 derived-state discard (Retained / BeforeKill / BetweenReopens /
//! BeforeClose) matrix AND full 1..=total exact-parked I/O-barrier sweep over
//! mixed Native + Signal + Bridge + World publications in fresh OS processes.
mod assertions;
mod child;
mod verify;
mod wire;

use self::assertions::{classify_signal_phase, SignalRef};
use self::child::SignalSession;
use self::verify::verify_signal_cell_reopen;
use self::wire::{SignalCmd, WireSnapshot};
use super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::opening;
use super::dimensions_matrix::{
    assert_cadence_checkpointed, run_parallel_items, BuildDimension, DerivedDimension,
    SeededCoverage,
};
use super::signal_bridge_io_kills::{
    build_signal_reference, kill_signal_exact, measure_signal_syncs, open_signal_home,
    publish_definition, SignalOpen,
};
use super::{
    assert_all_reopen_phases_killed, run_parallel_calibrated_barriers, ReopenPhase, RestorableHome,
    BUILD_PRIMARY,
};
use std::path::Path;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    WorthQueryApplicationHomeCloseOutcome as CloseOutcome,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::ContactCounters;
use worth_store_application::ApplicationHomeCheckpointDenial;

fn prepare_signal_base(
    loc: &Path,
    execution: &ExecutionRequest<'_, '_>,
    discard_first: bool,
) -> (Option<SeededCoverage>, Option<u64>) {
    let home = open_signal_home(
        SignalOpen {
            location: Location::Directory(loc.to_path_buf()),
            build: BUILD_PRIMARY,
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: ContactCounters::default(),
            placement: None,
        },
        execution,
    );
    let main = home.runtime().current_world();
    publish_definition(&home, main);
    home.close(execution).unwrap().reclaim();

    if discard_first {
        let home = open_signal_home(
            SignalOpen {
                location: Location::Directory(loc.to_path_buf()),
                build: BUILD_PRIMARY,
                initialized: Arc::new(AtomicUsize::new(0)),
                contacts: ContactCounters::default(),
                placement: None,
            },
            execution,
        );
        let (_, discard_gen) = checkpoint_and_discard_signal(&home, execution);
        let closed = home.close(execution).unwrap();
        assert!(matches!(
            closed.outcome(),
            CloseOutcome::Closed {
                checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                ..
            }
        ));
        closed.reclaim();
        (None, Some(discard_gen))
    } else {
        let ch = tempfile::tempdir().unwrap();
        let (mut s, opened) = SignalSession::start(loc, &ch.path().join("seed"), BUILD_PRIMARY);
        assert_cadence_checkpointed(&opened.cadence, "signal-seed");
        let cov = SeededCoverage {
            native_covered: opened.cadence["relational.canonical-commits"].0,
            world_covered: opened.cadence["world.occurrences"].0,
        };
        let c = s.send(SignalCmd::Close {
            expect_discarded: false,
        });
        assert!(c.closed);
        s.finish();
        (Some(cov), None)
    }
}

#[derive(Clone, Copy)]
struct PhaseOrdinals {
    total: u64,
    pre_op: u64,
    custody: u64,
    committed: u64,
}

impl PhaseOrdinals {
    fn for_phase(self, phase: ReopenPhase) -> u64 {
        match phase {
            ReopenPhase::PreOp => self.pre_op,
            ReopenPhase::Custody => self.custody,
            ReopenPhase::Committed => self.committed,
        }
    }
}

fn probe_signal_ordinal(
    saved: &RestorableHome,
    ord: u64,
    total: u64,
    probe_dir: &Path,
    expected_before: &WireSnapshot,
) -> ReopenPhase {
    saved.restore();
    kill_signal_exact(saved.location(), ord, total);
    let (mut s, opened) = SignalSession::start(saved.location(), probe_dir, BUILD_PRIMARY);
    let phase = classify_signal_phase(&opened, expected_before);
    s.kill_live();
    phase
}

fn discover_signal_phase_ordinals(
    saved: &RestorableHome,
    expected_before: &WireSnapshot,
) -> PhaseOrdinals {
    let (m1, m2) = (measure_signal_syncs(saved), measure_signal_syncs(saved));
    assert_eq!(
        m1, m2,
        "stabilized Signal fixture must calibrate identically"
    );
    let total = m1;
    let probe_dir = tempfile::tempdir().unwrap();
    let mut seq = 0usize;
    let mut probe = |ord: u64| {
        seq += 1;
        probe_signal_ordinal(
            saved,
            ord,
            total,
            &probe_dir.path().join(format!("s{seq}")),
            expected_before,
        )
    };
    let pre_op = (1..=total)
        .find(|&o| probe(o) == ReopenPhase::PreOp)
        .expect("Signal/Bridge must have a PreOp barrier kill");
    let hi_committed = (1..=total)
        .rev()
        .find(|&o| probe(o) == ReopenPhase::Committed)
        .expect("Signal/Bridge must have a Committed barrier kill");
    let (mut lo, mut hi) = (pre_op, hi_committed);
    let mut any_custody = None;
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        match probe(mid) {
            ReopenPhase::Committed => hi = mid,
            ReopenPhase::Custody => {
                any_custody = Some(mid);
                lo = mid;
            }
            ReopenPhase::PreOp => lo = mid,
        }
    }
    let first_committed = hi;
    let (mut c_lo, mut c_hi) = (pre_op, first_committed);
    while c_lo + 1 < c_hi {
        let mid = c_lo + (c_hi - c_lo) / 2;
        match probe(mid) {
            ReopenPhase::PreOp => c_lo = mid,
            ReopenPhase::Custody => {
                any_custody = Some(mid);
                c_hi = mid;
            }
            ReopenPhase::Committed => c_hi = mid,
        }
    }
    let custody = if c_hi < first_committed {
        let center = c_hi + (first_committed - 1 - c_hi) / 2;
        if probe(center) == ReopenPhase::Custody {
            Some(center)
        } else {
            any_custody
        }
    } else {
        any_custody
    }
    .or_else(|| (pre_op..first_committed).find(|&o| probe(o) == ReopenPhase::Custody))
    .expect("Signal/Bridge must have a Custody barrier kill");
    let committed = first_committed + (hi_committed - first_committed) / 2;
    assert_eq!(
        probe(committed),
        ReopenPhase::Committed,
        "interior committed"
    );
    PhaseOrdinals {
        total,
        pre_op,
        custody,
        committed,
    }
}

#[test]
fn signal_bridge_mixed_kills_across_build_change_and_d14_discard() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (rb, ra, target_key_identity, definition_identity) = build_signal_reference(&execution);
    let sig_ref = SignalRef {
        before: WireSnapshot::from_snapshot(&rb),
        after: WireSnapshot::from_snapshot(&ra),
        target_key_identity,
        definition_identity,
    };

    let retained_probe = RestorableHome::prepare(|loc| {
        prepare_signal_base(loc, &execution, false);
    });
    let discarded_probe = RestorableHome::prepare(|loc| {
        prepare_signal_base(loc, &execution, true);
    });
    let retained_ord = discover_signal_phase_ordinals(&retained_probe, &sig_ref.before);
    let discarded_ord = discover_signal_phase_ordinals(&discarded_probe, &sig_ref.before);

    let phases = [
        ReopenPhase::PreOp,
        ReopenPhase::Custody,
        ReopenPhase::Committed,
    ];
    let mut cells = Vec::with_capacity(36);
    for phase in phases {
        for build_dim in BuildDimension::ALL {
            for derived_dim in DerivedDimension::ALL {
                cells.push((phase, build_dim, derived_dim));
            }
        }
    }
    run_parallel_items(
        "C4b.3 Signal/Bridge matrix",
        &cells,
        |&(phase, build_dim, derived_dim)| {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            let discard_first = derived_dim == DerivedDimension::DiscardedBeforeKill;
            let (mut seeded, mut pre_gen) = (None, None);
            let base = RestorableHome::prepare(|loc| {
                (seeded, pre_gen) = prepare_signal_base(loc, &w_exec, discard_first);
            });
            let ord = if discard_first {
                discarded_ord
            } else {
                retained_ord
            };
            kill_signal_exact(base.location(), ord.for_phase(phase), ord.total);
            let label = format!("signal-matrix/{phase:?}/{build_dim:?}/{derived_dim:?}");
            let observed = verify_signal_cell_reopen(
                base.location(),
                (build_dim, derived_dim),
                (seeded, pre_gen),
                (&sig_ref, &label),
            );
            assert_eq!(observed, phase, "{label}: observed phase must match target");
        },
    );
}

#[test]
fn signal_bridge_all_barriers_across_build_change_and_d14_discard() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (rb, ra, target_key_identity, definition_identity) = build_signal_reference(&execution);
    let sig_ref = SignalRef {
        before: WireSnapshot::from_snapshot(&rb),
        after: WireSnapshot::from_snapshot(&ra),
        target_key_identity,
        definition_identity,
    };
    let outcomes = run_parallel_calibrated_barriers(
        "Signal/Bridge D14+Alternating",
        || {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            let mut dgen = None;
            let home = RestorableHome::prepare(|loc| {
                (_, dgen) = prepare_signal_base(loc, &w_exec, true);
            });
            let (m1, m2) = (measure_signal_syncs(&home), measure_signal_syncs(&home));
            assert_eq!(
                m1, m2,
                "stabilized D14 Signal layout must calibrate identically"
            );
            ((home, dgen), m1)
        },
        |(worker_home, dgen), ordinal, worker_total| {
            worker_home.restore();
            kill_signal_exact(worker_home.location(), ordinal, worker_total);
            let label = format!("signal-barriers-dims/sync#{ordinal}");
            verify_signal_cell_reopen(
                worker_home.location(),
                (
                    BuildDimension::Alternating,
                    DerivedDimension::DiscardedBeforeKill,
                ),
                (None, *dgen),
                (&sig_ref, &label),
            )
        },
    );
    assert_all_reopen_phases_killed("Signal/Bridge D14+Alternating", &outcomes);
}

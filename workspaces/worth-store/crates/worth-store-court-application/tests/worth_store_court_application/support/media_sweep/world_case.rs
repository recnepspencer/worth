//! Media-sweep case runner with 100% fault/recovery coverage and content-based
//! equivalence-class deduplication of the reopen-based `assert_continues_world`
//! tail.
//!
//! `Target::RegionLoad` continues directly on the first recovered
//! `WorldHomeFixture` (after its Native checkpoint) without dropping, reopening,
//! or deduplicating. `Commit`, `BranchTransition`, and `Checkpoint` run full
//! recovery and crash/transition oracles on every cut, abruptly drop the
//! worldless recovery handle, hash the exact quiescent on-disk bytes at
//! `location`, and run `assert_continues_world` once per distinct content-based
//! `PostRecoveryEquivalenceKey`.

use std::sync::atomic::Ordering;
use std::time::Instant;

use worth_execution::ExecutionRequest;
use worth_relational::facade::mvcc::RelationalTransactionReadDenial;
use worth_relational::facade::storage::RelationalStorageSubject;

use crate::crash_oracle::InFlight;
use crate::media_case::{Ended, Target};
use crate::media_faults::FaultedMedia;
use crate::media_sweep::equivalence::{build_equivalence_key, EquivalenceLedger};
use crate::media_sweep::prepared_sweep_home::PreparedSweepHome;
use crate::media_sweep::targets::PhaseTimings;
use crate::media_sweep::world_continuation::{
    assert_continues_world, finish_region_load_case, open_native_recovering,
    open_world_declaring_recovered, prepare_native_target_home, prepare_world_checkpoint_home,
};
use crate::media_sweep::SeedSweep;

/// Run `target` once on a restored prefix, execute 100% of fault injection,
/// recovery, and post-recovery assertions, and run `assert_continues_world`
/// directly for `RegionLoad` or once per distinct byte-exact
/// `PostRecoveryEquivalenceKey` for reopen-based targets.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_sweep_case(
    seed_sweep: &SeedSweep,
    prepared: &PreparedSweepHome,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
    ledger: &EquivalenceLedger,
    timings: &PhaseTimings,
    case: &str,
) -> Ended {
    let script = &seed_sweep.script;
    let target = seed_sweep.target;
    let (
        ended,
        acknowledged,
        in_flight,
        checkpoints,
        checkpoint_before,
        acknowledged_transitions,
        transition_flight,
    ) = match target {
        Target::Checkpoint(_) => {
            run_checkpoint_target(seed_sweep, prepared, media, execution, timings, case)
        }
        _ => run_native_target(seed_sweep, prepared, media, execution, timings, case),
    };

    if matches!(target, Target::RegionLoad(_)) {
        return finish_region_load_case(
            prepared,
            script,
            ended,
            &checkpoints,
            &acknowledged,
            &acknowledged_transitions,
            execution,
            ledger,
            timings,
            case,
        );
    }

    let t2 = Instant::now();
    let (recovered, recovered_schema) = open_native_recovering(prepared.location(), execution)
        .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
    timings
        .recover_us
        .fetch_add(t2.elapsed().as_micros() as u64, Ordering::Relaxed);

    let t3 = Instant::now();
    let selected = crate::selected_checkpoint::generation(&recovered);
    assert!(
        checkpoints.contains(&selected),
        "{case}: checkpoint {selected} selected, expected one of {checkpoints:?}"
    );
    let commits = crate::crash_oracle::assert_recovered(
        execution,
        &recovered,
        script,
        &acknowledged,
        &in_flight,
        case,
    );
    let transition_landed = crate::transition_oracle::assert_recovered(
        &recovered,
        &acknowledged_transitions,
        &transition_flight,
        case,
    );
    let landed = match target {
        Target::Commit(_) => commits > acknowledged.len(),
        Target::Checkpoint(_) => selected > checkpoint_before,
        Target::BranchTransition(_) => transition_landed,
        Target::RegionLoad(_) => unreachable!(),
    };
    timings
        .oracle_us
        .fetch_add(t3.elapsed().as_micros() as u64, Ordering::Relaxed);

    let t4 = Instant::now();
    let commit_artifacts =
        crate::workload::artifacts_of(execution, recovered.relational().runtime());
    let transition_artifacts = crate::branch_lifecycle::artifacts_of_home(&recovered);
    let logical_state =
        crate::branch_lifecycle::state_of(recovered.relational().runtime(), &script.records);
    let verified_physical = crate::media_case::verified_recovered_physical(&recovered);
    let final_ended = match ended {
        Ended::Unresolved { .. } => Ended::Unresolved { recovered: landed },
        other => other,
    };
    let effective_flight = match target {
        Target::BranchTransition(_) => &transition_flight,
        _ => &in_flight,
    };
    let key_base = verified_physical.map(|physical| {
        build_equivalence_key(
            recovered.relational().runtime(),
            physical,
            final_ended,
            effective_flight,
            recovered_schema,
            &commit_artifacts,
            &transition_artifacts,
            &logical_state,
            selected,
        )
    });

    // Deliberate abrupt drop: bypasses the close checkpoint and releases file
    // leases when handles drop so the quiescent on-disk layout is hashed at the
    // exact state `open_world_declaring_recovered` will actually open.
    drop(recovered);

    let (should_continue, continuation_guard) = match key_base {
        Some(base) => {
            let claimed =
                ledger.claim_continuation(base.with_quiescent_disk_layout(prepared.location()));
            (claimed.is_some(), claimed)
        }
        None => {
            // Content verification failed (e.g., damaged member or Store locked
            // for inspection): equivalence cannot be established, so run
            // continuation unconditionally.
            ledger.record_direct_continuation();
            (true, None)
        }
    };
    timings
        .eq_us
        .fetch_add(t4.elapsed().as_micros() as u64, Ordering::Relaxed);

    if should_continue {
        let t5 = Instant::now();
        let world_recovered =
            open_world_declaring_recovered(prepared.location(), recovered_schema, execution)
                .unwrap_or_else(|refusal| {
                    panic!("{case}: World reopen for continuation refused: {refusal:?}")
                });
        assert_continues_world(
            prepared.location(),
            world_recovered,
            script,
            commits,
            execution,
            case,
        );
        if let Some(guard) = continuation_guard {
            guard.mark_succeeded();
        }
        timings
            .continues_us
            .fetch_add(t5.elapsed().as_micros() as u64, Ordering::Relaxed);
    }
    final_ended
}

type TargetRunResult = (
    Ended,
    Vec<Vec<u8>>,
    InFlight,
    Vec<u64>,
    u64,
    Vec<Vec<u8>>,
    InFlight,
);

fn run_checkpoint_target(
    seed_sweep: &SeedSweep,
    prepared: &PreparedSweepHome,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
    timings: &PhaseTimings,
    case: &str,
) -> TargetRunResult {
    let lead = seed_sweep.lead();
    let t0 = Instant::now();
    let (fixture, activations, before, acknowledged_transitions) = prepare_world_checkpoint_home(
        seed_sweep,
        prepared.location(),
        prepared.schema(),
        media,
        execution,
    );
    let lead_checkpoints = lead
        .iter()
        .filter(|step| matches!(step, crate::workload::Step::Checkpoint))
        .count() as u64;
    let checkpoint_before = seed_sweep.prefix_checkpoint_generation + lead_checkpoints;
    timings
        .open_fault_us
        .fetch_add(t0.elapsed().as_micros() as u64, Ordering::Relaxed);

    let t1 = Instant::now();
    crate::media_faults::arm(&activations);
    let outcome = fixture.home.checkpoint(execution);
    assert!(
        activations[0].is_consumed(),
        "{case}: the fault did not fire"
    );
    let (ended, checkpoints) =
        crate::media_case::checkpoint_outcome(outcome, checkpoint_before, case);
    // Deliberate abrupt drop: bypasses close checkpoint to simulate crash.
    drop(fixture);
    timings
        .target_us
        .fetch_add(t1.elapsed().as_micros() as u64, Ordering::Relaxed);

    (
        ended,
        before,
        InFlight::Nothing,
        checkpoints,
        checkpoint_before,
        acknowledged_transitions,
        InFlight::Nothing,
    )
}

fn run_native_target(
    seed_sweep: &SeedSweep,
    prepared: &PreparedSweepHome,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
    timings: &PhaseTimings,
    case: &str,
) -> TargetRunResult {
    let script = &seed_sweep.script;
    let lead = seed_sweep.lead();
    let target = seed_sweep.target;
    let t0 = Instant::now();
    let (home, activations, before, mut acknowledged_transitions, transition_state_before) =
        prepare_native_target_home(
            seed_sweep,
            prepared.location(),
            prepared.schema(),
            media,
            execution,
        );
    let mut transition_flight = InFlight::Nothing;
    let checkpoint_before = if lead.is_empty() {
        seed_sweep.prefix_checkpoint_generation
    } else {
        crate::selected_checkpoint::generation(&home)
    };
    timings
        .open_fault_us
        .fetch_add(t0.elapsed().as_micros() as u64, Ordering::Relaxed);

    let t1 = Instant::now();
    crate::media_faults::arm(&activations);
    let (ended, acknowledged, in_flight, checkpoints) = match target {
        Target::Commit(index) => {
            let outcome = script.steps[index].commit(home.relational().runtime(), execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: the fault did not fire"
            );
            let (ended, acknowledged, in_flight) =
                crate::media_case::commit_outcome(&home, before, outcome, execution, case);
            (ended, acknowledged, in_flight, vec![checkpoint_before])
        }
        Target::RegionLoad(_) => {
            let before_commits = before.len();
            let outcome = crate::residency_media::read_first(&home, script, execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: region fault did not fire"
            );
            let ended = match outcome {
                Ok(rendered) => {
                    assert_eq!(
                        rendered,
                        crate::residency_media::expected_first(script, before_commits),
                        "{case}: faulted read differs from memory"
                    );
                    Ended::Acknowledged
                }
                Err(RelationalTransactionReadDenial::StorageUnavailable(failure)) => {
                    assert!(
                        matches!(
                            failure.subject,
                            RelationalStorageSubject::Region(_)
                                | RelationalStorageSubject::Symbols { .. }
                        ),
                        "{case}: {failure:?}"
                    );
                    Ended::Refused
                }
                Err(other) => panic!("{case}: region read returned {other:?}"),
            };
            let expected = &script.states[before_commits];
            assert_eq!(
                &crate::workload::states_of(home.relational().runtime(), &script.records),
                expected
            );
            (ended, before, InFlight::Nothing, vec![checkpoint_before])
        }
        Target::BranchTransition(index) => {
            let outcome = script.steps[index].lifecycle(home.relational().runtime(), execution);
            assert!(
                activations[0].is_consumed(),
                "{case}: transition fault did not fire"
            );
            if outcome.is_err() {
                assert_eq!(
                    Some(crate::branch_lifecycle::state_of(
                        home.relational().runtime(),
                        &script.records
                    )),
                    transition_state_before,
                    "{case}: stopped transition changed live state"
                );
            }
            let (ended, acknowledged, flight) = crate::transition_oracle::outcome(
                &home,
                &acknowledged_transitions,
                outcome,
                execution,
                case,
            );
            acknowledged_transitions = acknowledged;
            transition_flight = flight;
            (ended, before, InFlight::Nothing, vec![checkpoint_before])
        }
        Target::Checkpoint(_) => unreachable!("handled by run_checkpoint_target"),
    };
    // Deliberate abrupt drop: bypasses close checkpoint to simulate crash.
    drop(home);
    timings
        .target_us
        .fetch_add(t1.elapsed().as_micros() as u64, Ordering::Relaxed);

    (
        ended,
        acknowledged,
        in_flight,
        checkpoints,
        checkpoint_before,
        acknowledged_transitions,
        transition_flight,
    )
}

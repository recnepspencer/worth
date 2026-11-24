//! World-composed opening, continuation, and fresh-identity verification for
//! `media_sweep`, plus lightweight phase-timing instrumentation.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Instant;

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::transactions::RecordRef;
use worth_store::physical_runtime::certification::CertificationMediaFaultActivation;
use worth_store_application::{
    ApplicationHomeCloseOutcome, ReadmittedApplicationHome, RelationalHomeOpenRefused,
    WorldReadmittedApplicationHome,
};
use worth_store_testkit_application_oracle::facade::{
    home_opening,
    world_home::{WorldHomeFixture, WorldHomeFixtureRefusal},
};

use crate::branch_lifecycle::State;
use crate::media_case::{Ended, Target};
use crate::media_faults::FaultedMedia;
use crate::media_sweep::equivalence::EquivalenceLedger;
use crate::media_sweep::prepared_sweep_home::PreparedSweepHome;
use crate::media_sweep::targets::PhaseTimings;
use crate::media_sweep::SeedSweep;
use crate::workload::Script;

pub(crate) type WorldHome = WorldReadmittedApplicationHome<(), (), (), (), ()>;

pub(crate) fn open_world_with_faults(
    location: &Path,
    schema: SchemaVersionId,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
) -> (WorldHomeFixture, Vec<CertificationMediaFaultActivation>) {
    crate::media_case::open_world_with_media(location, schema, media, execution)
}

/// Shared pre-arm setup for `Target::Checkpoint` so the counter run (`measure_at`)
/// and every faulted case execute the exact same operations before `arm()`.
pub(crate) fn prepare_world_checkpoint_home(
    seed_sweep: &SeedSweep,
    location: &Path,
    schema: SchemaVersionId,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    WorldHomeFixture,
    Vec<CertificationMediaFaultActivation>,
    Vec<Vec<u8>>,
    Vec<Vec<u8>>,
) {
    let lead = seed_sweep.lead();
    let (fixture, activations) = open_world_with_faults(location, schema, media, execution);
    crate::durable_run::world::apply(&fixture.home, execution, lead);
    if seed_sweep.prefix > 0 {
        // Deferred cold-symbol replacement coverage: tracked by the ignored test
        // `cold_symbol_page_checkpoint_replacement_recovers_without_prewarm` until
        // `worth-relational` `dictionary_pages::capture` carries `prior` cold pages.
        let _ = crate::branch_lifecycle::state_of(
            fixture.home.relational().runtime(),
            &seed_sweep.script.records,
        );
    }
    let acknowledged_transitions = if lead.is_empty() {
        seed_sweep.prefix_transitions.clone()
    } else {
        crate::branch_lifecycle::artifacts_of_world(&fixture.home)
    };
    let before = if lead.is_empty() {
        seed_sweep.prefix_artifacts.clone()
    } else {
        crate::workload::artifacts_of(execution, fixture.home.relational().runtime())
    };
    if seed_sweep.prefix == 0 {
        assert_eq!(
            fixture.home.relational().cold_storage_reads().member_reads,
            0,
            "cold first checkpoint must not pre-warm cold Relational members"
        );
    }
    crate::media_case::drain_world_clean_frames(&fixture);
    (fixture, activations, before, acknowledged_transitions)
}

/// Shared pre-arm setup for `Commit`, `BranchTransition`, and `RegionLoad` so
/// the counter run (`measure_at`) and every faulted case execute the exact same
/// operations before `arm()`.
pub(crate) fn prepare_native_target_home(
    seed_sweep: &SeedSweep,
    location: &Path,
    schema: SchemaVersionId,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    ReadmittedApplicationHome,
    Vec<CertificationMediaFaultActivation>,
    Vec<Vec<u8>>,
    Vec<Vec<u8>>,
    Option<State>,
) {
    let lead = seed_sweep.lead();
    let (home, activations) = media.open(location, schema, execution);
    crate::durable_run::apply(&home, execution, lead);
    let acknowledged_transitions = if lead.is_empty() {
        seed_sweep.prefix_transitions.clone()
    } else {
        crate::branch_lifecycle::artifacts_of_home(&home)
    };
    let transition_state_before =
        matches!(seed_sweep.target, Target::BranchTransition(_)).then(|| {
            crate::branch_lifecycle::state_of(
                home.relational().runtime(),
                &seed_sweep.script.records,
            )
        });
    if matches!(seed_sweep.target, Target::RegionLoad(_)) {
        crate::residency_media::prepare_cold_media_read(&home);
    }
    let before = if lead.is_empty() {
        seed_sweep.prefix_artifacts.clone()
    } else {
        crate::workload::artifacts_of(execution, home.relational().runtime())
    };
    (
        home,
        activations,
        before,
        acknowledged_transitions,
        transition_state_before,
    )
}

/// Recover `Target::RegionLoad` directly into `WorldHomeFixture`, verify exact
/// checkpoint generation and next-position commit/transition fate probes before
/// checkpointing, and continue directly on that same first recovered owner.
#[allow(clippy::too_many_arguments)]
pub(crate) fn finish_region_load_case(
    prepared: &PreparedSweepHome,
    script: &Script,
    ended: Ended,
    checkpoints: &[u64],
    acknowledged: &[Vec<u8>],
    acknowledged_transitions: &[Vec<u8>],
    execution: &ExecutionRequest<'_, '_>,
    ledger: &EquivalenceLedger,
    timings: &PhaseTimings,
    case: &str,
) -> Ended {
    use worth_relational::facade::branch::RelationalBranchTransitionSequence;
    use worth_relational::facade::durability::{
        RelationalBranchTransitionArtifact, RelationalCanonicalCommitArtifact,
    };
    use worth_relational::facade::publication::PatchStreamPosition;
    use worth_store_relational::{RelationalStoreCommitFate, RelationalStoreTransitionFate};

    let t2 = Instant::now();
    let world_recovered =
        open_world_declaring_recovered(prepared.location(), prepared.schema(), execution)
            .unwrap_or_else(|refusal| panic!("{case}: World recovery refused: {refusal:?}"));
    timings
        .recover_us
        .fetch_add(t2.elapsed().as_micros() as u64, Ordering::Relaxed);

    let t3 = Instant::now();
    let selected = crate::media_case::world_checkpoint_generation(&world_recovered);
    assert!(
        checkpoints.contains(&selected),
        "{case}: checkpoint {selected} selected, expected one of {checkpoints:?}"
    );
    let recovered_commits =
        crate::workload::artifacts_of(execution, world_recovered.home.relational().runtime());
    assert_eq!(recovered_commits, acknowledged, "{case}: commits differ");
    crate::durable_run::assert_reference_artifacts(&recovered_commits, script, case);
    if let Some(last) = acknowledged.last() {
        let probe = RelationalCanonicalCommitArtifact::from_untrusted_bytes(last.clone())
            .expect("commit artifact frame")
            .payload_digest();
        let next = acknowledged.len() as u64 + 1;
        let resolution = world_recovered
            .home
            .relational()
            .resolve(PatchStreamPosition(next), probe)
            .unwrap_or_else(|refusal| panic!("{case}: resolve {next}: {refusal:?}"));
        assert_eq!(
            resolution,
            RelationalStoreCommitFate::NotCommitted {
                head: acknowledged.len() as u64,
            },
            "{case}: resolve {next}"
        );
    }
    let recovered_transitions = crate::branch_lifecycle::artifacts_of_world(&world_recovered.home);
    assert_eq!(
        recovered_transitions, acknowledged_transitions,
        "{case}: transitions differ"
    );
    if let Some(last) = acknowledged_transitions.last() {
        let probe = RelationalBranchTransitionArtifact::from_untrusted_bytes(last.clone())
            .expect("transition artifact frame")
            .payload_digest();
        let next = RelationalBranchTransitionSequence(acknowledged_transitions.len() as u64 + 1);
        let resolution = world_recovered
            .home
            .relational()
            .resolve_branch_transition(next, probe)
            .unwrap_or_else(|refusal| panic!("{case}: resolve transition: {refusal:?}"));
        assert_eq!(
            resolution,
            RelationalStoreTransitionFate::NotCommitted {
                head: acknowledged_transitions.len() as u64,
            },
            "{case}: resolve transition"
        );
    }
    let commits = recovered_commits.len();
    crate::durable_run::world::assert_holds(
        execution,
        &world_recovered.home,
        script,
        &recovered_commits,
    );
    world_recovered
        .home
        .relational()
        .runtime()
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    crate::durable_run::world::assert_holds(
        execution,
        &world_recovered.home,
        script,
        &recovered_commits,
    );
    timings
        .oracle_us
        .fetch_add(t3.elapsed().as_micros() as u64, Ordering::Relaxed);

    let t5 = Instant::now();
    assert_continues_world(
        prepared.location(),
        world_recovered,
        script,
        commits,
        execution,
        case,
    );
    ledger.record_direct_continuation();
    timings
        .continues_us
        .fetch_add(t5.elapsed().as_micros() as u64, Ordering::Relaxed);

    match ended {
        Ended::Unresolved { .. } => Ended::Unresolved { recovered: false },
        other => other,
    }
}

pub(crate) fn open_world_declaring_recovered(
    location: &Path,
    schema: SchemaVersionId,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldHomeFixture, WorldHomeFixtureRefusal> {
    crate::durable_run::world::open(location, schema, false, execution)
}

pub(crate) fn open_native_recovering(
    location: &Path,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(ReadmittedApplicationHome, SchemaVersionId), home_opening::HomeOpeningRefused> {
    match crate::durable_run::open_declaring(location, crate::schema_transition::SOURCE, execution)
    {
        Ok(home) => Ok((home, crate::schema_transition::SOURCE)),
        Err(refusal)
            if crate::durable_run::recorded_schema(&refusal)
                == Some(crate::schema_transition::TARGET) =>
        {
            match refusal {
                home_opening::HomeOpeningRefused::Readmission(
                    RelationalHomeOpenRefused::Readmission(refused),
                ) => match refused.into_home().close(execution) {
                    Ok(ApplicationHomeCloseOutcome::Closed { .. }) => {
                        crate::durable_run::open_declaring(
                            location,
                            crate::schema_transition::TARGET,
                            execution,
                        )
                        .map(|home| (home, crate::schema_transition::TARGET))
                    }
                    Ok(outcome) => Err(home_opening::HomeOpeningRefused::Close(outcome)),
                    Err(home) => Err(home_opening::HomeOpeningRefused::UnrecoveredClose(home)),
                },
                other => Err(other),
            }
        }
        Err(other) => Err(other),
    }
}

pub(crate) fn assert_continues_world(
    location: &Path,
    fixture: WorldHomeFixture,
    script: &Script,
    recovered: usize,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) {
    let before = crate::workload::artifacts_of(execution, fixture.home.relational().runtime());
    let transitions = crate::branch_lifecycle::artifacts_of_world(&fixture.home).len();
    crate::durable_run::world::apply(
        &fixture.home,
        execution,
        crate::branch_lifecycle::steps_after(script, recovered, transitions),
    );
    let everything = crate::workload::artifacts_of(execution, fixture.home.relational().runtime());
    assert!(
        everything[..recovered] == before[..],
        "{case}: history moved"
    );
    crate::durable_run::world::assert_holds(execution, &fixture.home, script, &everything);
    crate::durable_run::world::close(fixture, execution);
    let again = crate::durable_run::world::open(
        location,
        crate::durable_run::served_schema(&script.steps),
        false,
        execution,
    )
    .unwrap_or_else(|refusal| panic!("{case}: reopen: {refusal:?}"));
    crate::durable_run::world::assert_holds(execution, &again.home, script, &everything);
    assert_creates_are_fresh_world(&again.home, script, execution, case);
    crate::durable_run::world::close(again, execution);
}

fn assert_creates_are_fresh_world(
    home: &WorldHome,
    script: &Script,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) {
    let committed: BTreeSet<&RecordRef> = script.records.iter().collect();
    assert!(
        crate::identity_safety::deletes_a_record(script),
        "{case}: the script's history must include a deleted record"
    );
    let runtime = home.relational().runtime();
    let mut step = crate::identity_safety::FIRST_FRESH_STEP;
    for name in ["main", "feature"] {
        let branch = BranchId(name.to_owned());
        if runtime.branch_identity(&branch).is_err() {
            continue;
        }
        let mut created = Vec::new();
        for _ in 0..2 {
            let outcome =
                crate::workload::commit(runtime, execution, &branch, crate::batches::create(step))
                    .unwrap_or_else(|error| panic!("{case}: fresh create on {name}: {error:?}"));
            created.extend(outcome.changed_records.iter().cloned());
            runtime
                .snapshots()
                .release_snapshot(&outcome.snapshot)
                .unwrap();
            step += 1;
        }
        let entities: Vec<_> = created
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(entity) => Some(*entity),
                RecordRef::Relation(_) => None,
            })
            .collect();
        let [source, target] = entities[..] else {
            panic!("{case}: two creates on {name} issued {entities:?}");
        };
        let outcome = crate::workload::commit(
            runtime,
            execution,
            &branch,
            crate::batches::relate(source, target, step),
        )
        .unwrap_or_else(|error| panic!("{case}: fresh relation on {name}: {error:?}"));
        created.extend(outcome.changed_records.iter().cloned());
        runtime
            .snapshots()
            .release_snapshot(&outcome.snapshot)
            .unwrap();
        step += 1;
        for record in &created {
            assert!(
                !committed.contains(record),
                "{case}: {name} reissued the committed identity {record:?}"
            );
        }
    }
}

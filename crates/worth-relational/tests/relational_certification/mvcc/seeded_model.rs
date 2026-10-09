use std::panic::{catch_unwind, AssertUnwindSafe};

use super::world::supply_chain::{DeltaId, OracleBranch, ScaleName};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::snapshots::SnapshotHandle;

#[path = "seeded_trace_execution.rs"]
mod seeded_trace_execution;
#[path = "seeded_trace_lifecycle.rs"]
mod seeded_trace_lifecycle;
#[path = "seeded_trace_observation.rs"]
mod seeded_trace_observation;
#[path = "seeded_trace_shrinking.rs"]
mod seeded_trace_shrinking;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScenarioLifecycle {
    CommitOnly,
    RetainArchiveObserveRelease,
    DeleteAfterCommit,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum OutcomeProbe {
    ObserveCurrentBranch,
    CancelBeforeObservation,
    ReuseStaleBasis,
    RepeatAcceptedDelta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BranchScenario {
    delta: DeltaId,
    probe: OutcomeProbe,
    lifecycle: ScenarioLifecycle,
    expected_outcome: ProbeOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProbeOutcome {
    CurrentBranchObserved,
    ObservationCancelled,
    StaleBasisDenied,
    DuplicateDeltaDenied,
}

#[derive(Clone, Debug)]
struct ProductionModelTrace {
    profile: ScaleName,
    seed: u64,
    scenarios: Vec<BranchScenario>,
}

#[derive(Clone)]
struct BranchModelState {
    branch: BranchId,
    scenario: BranchScenario,
    oracle: OracleBranch,
    retained_snapshot: Option<SnapshotHandle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FailureSite {
    delta: DeltaId,
    probe: OutcomeProbe,
    expected_outcome: ProbeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TraceFailureIdentity {
    Operation {
        site: Option<FailureSite>,
        operation: &'static str,
        failure_class: String,
    },
    UnexpectedPanic(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TraceFailure {
    identity: TraceFailureIdentity,
    observation: String,
}

#[test]
fn generated_standard_sequences_cover_all_deltas_and_probes() {
    let mut covered_deltas = std::collections::BTreeSet::new();
    let mut covered_probes = std::collections::BTreeSet::new();
    for sequence in 0..8_u64 {
        let trace = generate_trace(standard_sequence_seed(sequence));
        covered_deltas.extend(trace.scenarios.iter().map(|scenario| scenario.delta));
        covered_probes.extend(trace.scenarios.iter().map(|scenario| scenario.probe));
    }
    assert_eq!(covered_deltas, DeltaId::ALL.into_iter().collect());
    assert_eq!(covered_probes.len(), 4);
}

fn standard_sequence_seed(sequence: u64) -> u64 {
    0x9171_1200_u64.wrapping_add(sequence.wrapping_mul(0x9e37_79b9))
}

fn assert_standard_sequence_replayable_and_shrinkable(sequence: u64) {
    let trace = generate_trace(standard_sequence_seed(sequence));
    if let Err(failure) = replay_trace(&trace) {
        let (shrunk, shrunk_failure) =
            seeded_trace_shrinking::shrink_failing_trace(trace.clone(), failure.clone());
        panic!(
            "generated production model failed\noriginal={trace:#?}\n\
             original_failure={failure:#?}\nshrunk={shrunk:#?}\n\
             shrunk_failure={shrunk_failure:#?}"
        );
    }
}

#[test]
fn generated_standard_seed_91711200_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(0);
}

#[test]
fn generated_standard_seed_12fa88bb9_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(1);
}

#[test]
fn generated_standard_seed_1cde00572_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(2);
}

#[test]
fn generated_standard_seed_26c177f2b_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(3);
}

#[test]
fn generated_standard_seed_30a4ef8e4_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(4);
}

#[test]
fn generated_standard_seed_3a886729d_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(5);
}

#[test]
fn generated_standard_seed_446bdec56_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(6);
}

#[test]
fn generated_standard_seed_4e4f5660f_is_replayable_and_shrinkable() {
    assert_standard_sequence_replayable_and_shrinkable(7);
}

fn generate_trace(seed: u64) -> ProductionModelTrace {
    let mut deltas = shuffled_v1_deltas(seed);
    deltas.truncate(3 + seed as usize % 5);
    deltas.push(DeltaId::AdoptHazardClassificationV2);
    let scenarios = deltas
        .iter()
        .enumerate()
        .map(|(index, delta)| {
            let posture_index = if *delta == DeltaId::AdoptHazardClassificationV2 {
                2
            } else {
                index % 4
            };
            let (probe, expected_outcome) = match posture_index {
                0 => (
                    OutcomeProbe::ObserveCurrentBranch,
                    ProbeOutcome::CurrentBranchObserved,
                ),
                1 => (
                    OutcomeProbe::CancelBeforeObservation,
                    ProbeOutcome::ObservationCancelled,
                ),
                2 => (
                    OutcomeProbe::ReuseStaleBasis,
                    ProbeOutcome::StaleBasisDenied,
                ),
                _ => (
                    OutcomeProbe::RepeatAcceptedDelta,
                    ProbeOutcome::DuplicateDeltaDenied,
                ),
            };
            BranchScenario {
                delta: *delta,
                probe,
                expected_outcome,
                lifecycle: if index == 0 {
                    ScenarioLifecycle::RetainArchiveObserveRelease
                } else if index + 1 == deltas.len() {
                    ScenarioLifecycle::DeleteAfterCommit
                } else {
                    ScenarioLifecycle::CommitOnly
                },
            }
        })
        .collect();
    ProductionModelTrace {
        profile: ScaleName::Standard,
        seed,
        scenarios,
    }
}

fn replay_trace(trace: &ProductionModelTrace) -> Result<(), TraceFailure> {
    let (mut world, _) = super::world::supply_chain::certified_supply_chain_world(
        super::world::supply_chain::SupplyChainScale::standard(),
    );
    catch_unwind(AssertUnwindSafe(|| {
        seeded_trace_execution::execute_trace(&mut world, trace)
    }))
    .map_err(panic_observation)
}

fn trace_panic(
    trace: &ProductionModelTrace,
    scenario: usize,
    operation: &'static str,
    error: impl std::fmt::Debug,
) -> ! {
    let failure_class = failure_class(&error);
    std::panic::panic_any(TraceFailure {
        identity: TraceFailureIdentity::Operation {
            site: trace
                .scenarios
                .get(scenario)
                .copied()
                .map(|scenario| FailureSite {
                    delta: scenario.delta,
                    probe: scenario.probe,
                    expected_outcome: scenario.expected_outcome,
                }),
            operation,
            failure_class,
        },
        observation: format!(
            "trace={trace:#?} scenario={scenario} operation={operation} error={error:?}"
        ),
    })
}

fn panic_observation(payload: Box<dyn std::any::Any + Send>) -> TraceFailure {
    let payload = match payload.downcast::<TraceFailure>() {
        Ok(failure) => return *failure,
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<String>() {
        Ok(text) => {
            let identity = TraceFailureIdentity::UnexpectedPanic((*text).clone());
            return TraceFailure {
                identity,
                observation: *text,
            };
        }
        Err(payload) => payload,
    };
    let observation = payload.downcast_ref::<&str>().map_or_else(
        || "non-text canonical panic".to_owned(),
        |text| (*text).to_owned(),
    );
    TraceFailure {
        identity: TraceFailureIdentity::UnexpectedPanic(observation.clone()),
        observation,
    }
}

fn failure_class(error: &impl std::fmt::Debug) -> String {
    let rendered = format!("{error:?}");
    let discriminators = rendered
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| token.chars().next().is_some_and(char::is_uppercase))
        .take(2)
        .collect::<Vec<_>>()
        .join("/");
    format!(
        "{}::{}",
        std::any::type_name_of_val(error),
        if discriminators.is_empty() {
            "unknown"
        } else {
            &discriminators
        }
    )
}

fn branch_id(delta: DeltaId) -> BranchId {
    let name = match delta {
        DeltaId::StormRerouteAurora => "storm",
        DeltaId::MaintainAtlasBerth => "maintenance",
        DeltaId::HoldMedicalCargo => "medical-hold",
        DeltaId::ExpandSouthpointCapacity => "southpoint-expansion",
        DeltaId::CompetingAuroraArrival => "competing-arrival",
        DeltaId::RetireAtlasWhileInspectingAurora => "inspection",
        DeltaId::RewireAuroraPortCall => "rewire",
        DeltaId::AdoptHazardClassificationV2 => "hazard-v2",
    };
    BranchId(name.to_owned())
}

fn shuffled_v1_deltas(mut state: u64) -> Vec<DeltaId> {
    let mut deltas = vec![
        DeltaId::StormRerouteAurora,
        DeltaId::MaintainAtlasBerth,
        DeltaId::HoldMedicalCargo,
        DeltaId::ExpandSouthpointCapacity,
        DeltaId::CompetingAuroraArrival,
        DeltaId::RetireAtlasWhileInspectingAurora,
        DeltaId::RewireAuroraPortCall,
    ];
    for upper in (1..deltas.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        deltas.swap(upper, state as usize % (upper + 1));
    }
    deltas
}

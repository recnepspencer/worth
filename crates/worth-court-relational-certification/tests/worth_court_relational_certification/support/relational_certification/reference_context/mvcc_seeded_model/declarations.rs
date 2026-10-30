pub(crate) use {std::panic::catch_unwind, std::panic::AssertUnwindSafe};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::snapshots::SnapshotHandle};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScenarioLifecycle {
    CommitOnly,
    RetainArchiveObserveRelease,
    DeleteAfterCommit,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OutcomeProbe {
    ObserveCurrentBranch,
    CancelBeforeObservation,
    ReuseStaleBasis,
    RepeatAcceptedDelta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BranchScenario {
    pub(crate) delta: DeltaId,
    pub(crate) probe: OutcomeProbe,
    pub(crate) lifecycle: ScenarioLifecycle,
    pub(crate) expected_outcome: ProbeOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProbeOutcome {
    CurrentBranchObserved,
    ObservationCancelled,
    StaleBasisDenied,
    DuplicateDeltaDenied,
}

#[derive(Clone, Debug)]
pub(crate) struct ProductionModelTrace {
    pub(crate) profile: ScaleName,
    pub(crate) seed: u64,
    pub(crate) scenarios: Vec<BranchScenario>,
}

#[derive(Clone)]
pub(crate) struct BranchModelState {
    pub(crate) branch: BranchId,
    pub(crate) scenario: BranchScenario,
    pub(crate) oracle: OracleBranch,
    pub(crate) retained_snapshot: Option<SnapshotHandle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FailureSite {
    pub(crate) delta: DeltaId,
    pub(crate) probe: OutcomeProbe,
    pub(crate) expected_outcome: ProbeOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TraceFailureIdentity {
    Operation {
        site: Option<FailureSite>,
        operation: &'static str,
        failure_class: String,
    },
    UnexpectedPanic(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TraceFailure {
    pub(crate) identity: TraceFailureIdentity,
    pub(crate) observation: String,
}

pub(crate) fn standard_sequence_seed(sequence: u64) -> u64 {
    0x9171_1200_u64.wrapping_add(sequence.wrapping_mul(0x9e37_79b9))
}

pub(crate) fn assert_standard_sequence_replayable_and_shrinkable(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    sequence: u64,
) {
    let trace = generate_trace(standard_sequence_seed(sequence));
    if let Err(failure) = replay_trace(execution, &trace) {
        let (shrunk, shrunk_failure) =
            crate::relational_certification::reference_context::mvcc_seeded_model::seeded_trace_shrinking::shrink_failing_trace(execution, trace.clone(), failure.clone());
        panic!(
            "generated production model failed\noriginal={trace:#?}\n\
             original_failure={failure:#?}\nshrunk={shrunk:#?}\n\
             shrunk_failure={shrunk_failure:#?}"
        );
    }
}

pub(crate) fn generate_trace(seed: u64) -> ProductionModelTrace {
    let mut deltas = shuffled_v1_deltas(seed);
    deltas.truncate(3 + seed as usize % 5);
    deltas.push(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2);
    let scenarios = deltas
        .iter()
        .enumerate()
        .map(|(index, delta)| {
            let posture_index = if *delta == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 {
                2
            } else {
                index % 4
            };
            let (probe, expected_outcome) = match posture_index {
                0 => (
                    crate::relational_certification::reference_context::mvcc_seeded_model::OutcomeProbe::ObserveCurrentBranch,
                    crate::relational_certification::reference_context::mvcc_seeded_model::ProbeOutcome::CurrentBranchObserved,
                ),
                1 => (
                    crate::relational_certification::reference_context::mvcc_seeded_model::OutcomeProbe::CancelBeforeObservation,
                    crate::relational_certification::reference_context::mvcc_seeded_model::ProbeOutcome::ObservationCancelled,
                ),
                2 => (
                    crate::relational_certification::reference_context::mvcc_seeded_model::OutcomeProbe::ReuseStaleBasis,
                    crate::relational_certification::reference_context::mvcc_seeded_model::ProbeOutcome::StaleBasisDenied,
                ),
                _ => (
                    crate::relational_certification::reference_context::mvcc_seeded_model::OutcomeProbe::RepeatAcceptedDelta,
                    crate::relational_certification::reference_context::mvcc_seeded_model::ProbeOutcome::DuplicateDeltaDenied,
                ),
            };
            BranchScenario {
                delta: *delta,
                probe,
                expected_outcome,
                lifecycle: if index == 0 {
                    crate::relational_certification::reference_context::mvcc_seeded_model::ScenarioLifecycle::RetainArchiveObserveRelease
                } else if index + 1 == deltas.len() {
                    crate::relational_certification::reference_context::mvcc_seeded_model::ScenarioLifecycle::DeleteAfterCommit
                } else {
                    crate::relational_certification::reference_context::mvcc_seeded_model::ScenarioLifecycle::CommitOnly
                },
            }
        })
        .collect();
    ProductionModelTrace {
        profile: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Standard,
        seed,
        scenarios,
    }
}

pub(crate) fn replay_trace(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    trace: &ProductionModelTrace,
) -> Result<(), TraceFailure> {
    let (mut world, _) = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world(
        execution,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::standard(),
    );
    catch_unwind(AssertUnwindSafe(|| {
        crate::relational_certification::reference_context::mvcc_seeded_model::seeded_trace_execution::execute_trace(execution, &mut world, trace)
    }))
    .map_err(panic_observation)
}

pub(crate) fn trace_panic(
    trace: &ProductionModelTrace,
    scenario: usize,
    operation: &'static str,
    error: impl std::fmt::Debug,
) -> ! {
    let failure_class = failure_class(&error);
    std::panic::panic_any(TraceFailure {
        identity: crate::relational_certification::reference_context::mvcc_seeded_model::TraceFailureIdentity::Operation {
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

pub(crate) fn panic_observation(payload: Box<dyn std::any::Any + Send>) -> TraceFailure {
    let payload = match payload.downcast::<TraceFailure>() {
        Ok(failure) => return *failure,
        Err(payload) => payload,
    };
    let payload = match payload.downcast::<String>() {
        Ok(text) => {
            let identity = crate::relational_certification::reference_context::mvcc_seeded_model::TraceFailureIdentity::UnexpectedPanic((*text).clone());
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
        identity: crate::relational_certification::reference_context::mvcc_seeded_model::TraceFailureIdentity::UnexpectedPanic(observation.clone()),
        observation,
    }
}

pub(crate) fn failure_class(error: &impl std::fmt::Debug) -> String {
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

pub(crate) fn branch_id(delta: DeltaId) -> BranchId {
    let name = match delta {
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora => "storm",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth => "maintenance",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo => "medical-hold",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity => "southpoint-expansion",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival => "competing-arrival",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora => "inspection",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall => "rewire",
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2 => "hazard-v2",
    };
    BranchId(name.to_owned())
}

pub(crate) fn shuffled_v1_deltas(mut state: u64) -> Vec<DeltaId> {
    let mut deltas = vec![
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::ExpandSouthpointCapacity,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::CompetingAuroraArrival,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RetireAtlasWhileInspectingAurora,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall,
    ];
    for upper in (1..deltas.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        deltas.swap(upper, state as usize % (upper + 1));
    }
    deltas
}

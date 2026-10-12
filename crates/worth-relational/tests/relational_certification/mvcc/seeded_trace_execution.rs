use std::collections::BTreeSet;

use super::{
    branch_id, trace_panic, BranchModelState, BranchScenario, OutcomeProbe, ProbeOutcome,
    ProductionModelTrace, ScenarioLifecycle,
};
use crate::world::supply_chain::{
    commit_branch_batch_with_result, commit_supply_chain_delta,
    lower_supply_chain_production_delta, OracleBranch, OracleState,
    ProductionSeededSupplyChainWorld, SupplyChainProductionDeltaLoweringError,
};
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalCancellationSource,
    RelationalOperationControl, RelationalTransactionIntent,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

pub(super) fn execute_trace(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
) {
    validate_trace(trace);
    let mut branches = Vec::<BranchModelState>::new();
    let mut parent_branch = BranchId("main".to_owned());
    let mut parent_oracle =
        OracleBranch::genesis(OracleState::from_definition(world.program.definition()));

    for (scenario_index, scenario) in trace.scenarios.iter().copied().enumerate() {
        let child_oracle = parent_oracle
            .fork(scenario.delta.branch(), parent_oracle.ancestry.branch)
            .unwrap_or_else(|error| trace_panic(trace, scenario_index, "oracle fork", error));
        let branch = fork_and_observe(
            execution,
            world,
            trace,
            scenario_index,
            scenario,
            &parent_branch,
            &child_oracle,
        );
        let committed_oracle = child_oracle
            .apply(scenario.delta)
            .unwrap_or_else(|error| trace_panic(trace, scenario_index, "oracle apply", error));
        commit_evolve_and_probe(
            execution,
            world,
            trace,
            scenario_index,
            scenario,
            &branch,
            &parent_oracle,
            &committed_oracle,
        );
        branches.push(BranchModelState {
            branch: branch.clone(),
            scenario,
            oracle: committed_oracle.clone(),
            retained_snapshot: None,
        });
        parent_branch = branch;
        parent_oracle = committed_oracle;
    }

    let archive_index = branches
        .iter()
        .position(|state| {
            state.scenario.lifecycle == ScenarioLifecycle::RetainArchiveObserveRelease
        })
        .expect("validated trace owns one archive scenario");
    let retained = super::seeded_trace_lifecycle::archive_and_observe(
        execution,
        world,
        trace,
        archive_index,
        &branches[archive_index],
    );
    branches[archive_index].retained_snapshot = Some(retained);

    let delete_index = branches
        .iter()
        .position(|state| state.scenario.lifecycle == ScenarioLifecycle::DeleteAfterCommit)
        .expect("validated trace owns one delete scenario");
    super::seeded_trace_lifecycle::delete_and_prove_absence(
        execution,
        world,
        trace,
        delete_index,
        &branches,
    );
    for state in &branches {
        if let Some(snapshot) = &state.retained_snapshot {
            world
                .runtime
                .snapshots()
                .release_snapshot(snapshot)
                .unwrap_or_else(|error| {
                    trace_panic(trace, archive_index, "archive release", error)
                });
        }
    }
}

fn validate_trace(trace: &ProductionModelTrace) {
    let shape_is_valid = trace.profile == crate::world::supply_chain::ScaleName::Standard
        && trace.seed != 0
        && trace.scenarios.len() >= 2
        && trace.scenarios.last().map(|scenario| scenario.delta)
            == Some(crate::world::supply_chain::DeltaId::AdoptHazardClassificationV2)
        && trace.scenarios.first().map(|scenario| scenario.lifecycle)
            == Some(ScenarioLifecycle::RetainArchiveObserveRelease)
        && trace.scenarios.last().map(|scenario| scenario.lifecycle)
            == Some(ScenarioLifecycle::DeleteAfterCommit)
        && trace.scenarios[1..trace.scenarios.len() - 1]
            .iter()
            .all(|scenario| scenario.lifecycle == ScenarioLifecycle::CommitOnly)
        && trace
            .scenarios
            .iter()
            .all(|scenario| expected_outcome(scenario.probe) == scenario.expected_outcome);
    let unique = trace
        .scenarios
        .iter()
        .map(|scenario| scenario.delta)
        .collect::<BTreeSet<_>>();
    if !shape_is_valid || unique.len() != trace.scenarios.len() {
        trace_panic(trace, 0, "trace validation", "invalid trace envelope");
    }
}

fn expected_outcome(probe: OutcomeProbe) -> ProbeOutcome {
    match probe {
        OutcomeProbe::ObserveCurrentBranch => ProbeOutcome::CurrentBranchObserved,
        OutcomeProbe::CancelBeforeObservation => ProbeOutcome::ObservationCancelled,
        OutcomeProbe::ReuseStaleBasis => ProbeOutcome::StaleBasisDenied,
        OutcomeProbe::RepeatAcceptedDelta => ProbeOutcome::DuplicateDeltaDenied,
    }
}

fn fork_and_observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    scenario: BranchScenario,
    parent: &BranchId,
    expected: &OracleBranch,
) -> BranchId {
    let branch = branch_id(scenario.delta);
    let (_, source) = world
        .runtime
        .observe_fork_source(parent)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "fork source", error));
    let fork = world
        .runtime
        .fork_branch(branch.clone(), source)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "fork", error));
    if fork.fork_provenance() != fork.source_observation() {
        trace_panic(trace, scenario_index, "fork provenance", &fork);
    }
    let reference = world
        .runtime
        .branch_reference_state(&branch)
        .unwrap_or_else(|| trace_panic(trace, scenario_index, "fork reference", &branch));
    if reference.fork_provenance() != Some(fork.source_observation()) {
        trace_panic(
            trace,
            scenario_index,
            "fork reference provenance",
            reference,
        );
    }
    observe_branch_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &branch,
        expected,
        "fork oracle",
    );
    branch
}

fn commit_evolve_and_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    scenario: BranchScenario,
    branch: &BranchId,
    parent_oracle: &OracleBranch,
    expected: &OracleBranch,
) {
    let applied = parent_oracle
        .ancestry
        .history
        .iter()
        .map(|event| event.delta)
        .collect::<BTreeSet<_>>();
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        branch,
        &applied,
        scenario.delta,
    )
    .unwrap_or_else(|error| trace_panic(trace, scenario_index, "lower", error));
    let committed = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        scenario.delta,
        batch,
    );
    super::seeded_trace_observation::observe_snapshot_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &committed.snapshot,
        expected,
        "oracle",
    );
    let identity = world
        .runtime
        .branch_identity(branch)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "identity", error));
    let stale_basis = world
        .runtime
        .admit_branch_basis(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "probe basis", error));
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "snapshot release", error));

    let successor = commit_branch_batch_with_result(
        execution,
        &world.runtime,
        branch.clone(),
        WorkerIntentBatch::new(format!("seeded-successor-{scenario_index}")),
    );
    super::seeded_trace_observation::observe_snapshot_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &successor.snapshot,
        expected,
        "successor oracle",
    );
    run_expected_probe(
        execution,
        world,
        trace,
        scenario_index,
        scenario,
        branch,
        &identity,
        stale_basis,
        &applied,
        expected,
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&successor.snapshot)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "successor release", error));
    let retention = world
        .runtime
        .branch_retention_cost_counters(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "retention", error));
    if retention.candidate_acquires != retention.candidate_releases {
        trace_panic(trace, scenario_index, "candidate release", retention);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_expected_probe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    scenario: BranchScenario,
    branch: &BranchId,
    identity: &worth_relational::facade::branch::RelationalBranchIdentity,
    stale_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    prior_applied: &BTreeSet<crate::world::supply_chain::DeltaId>,
    expected: &OracleBranch,
) {
    match scenario.probe {
        OutcomeProbe::ObserveCurrentBranch => observe_branch_against_oracle(
            execution,
            world,
            trace,
            scenario_index,
            branch,
            expected,
            "uninterrupted probe",
        ),
        OutcomeProbe::CancelBeforeObservation => {
            let source = RelationalCancellationSource::new();
            source.cancel();
            let control: RelationalOperationControl = source.token().into();
            let denial = world
                .runtime
                .observe_branch_with_control(identity, &control);
            if denial != Err(RelationalBranchBasisDenial::Cancelled) {
                trace_panic(trace, scenario_index, "cancelled observation", denial);
            }
        }
        OutcomeProbe::ReuseStaleBasis => {
            let denial = world
                .runtime
                .begin_branch_transaction(&stale_basis, RelationalTransactionIntent::ordinary());
            if !matches!(
                denial,
                Err(RelationalBranchTransactionAdmissionDenial::StaleBasis)
            ) {
                trace_panic(trace, scenario_index, "stale basis denial", denial);
            }
        }
        OutcomeProbe::RepeatAcceptedDelta => {
            let mut applied = prior_applied.clone();
            applied.insert(scenario.delta);
            let denial = lower_supply_chain_production_delta(
                execution,
                &world.runtime,
                &world.program,
                &world.handles,
                branch,
                &applied,
                scenario.delta,
            );
            if denial
                != Err(SupplyChainProductionDeltaLoweringError::DuplicateDelta(
                    scenario.delta,
                ))
            {
                trace_panic(trace, scenario_index, "duplicate delta denial", denial);
            }
        }
    }
}

fn observe_branch_against_oracle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    branch: &BranchId,
    expected: &OracleBranch,
    operation: &'static str,
) {
    let identity = world
        .runtime
        .branch_identity(branch)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
    let (_, basis) = world
        .runtime
        .observe_branch(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
    super::seeded_trace_observation::observe_snapshot_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &snapshot,
        expected,
        operation,
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error));
}

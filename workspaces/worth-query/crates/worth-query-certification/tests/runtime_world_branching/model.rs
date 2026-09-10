use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use worth_query_host::facade::{primary_graph, product::WorthQueryProductBranch, runtime};

use crate::adapters::ReplacementPredicate;
use crate::product_query_support::{
    fork_independent_product, fork_relational_product, reuse_exact_product,
};
use crate::query_probe::{assert_ordinary_work, assert_retained_work, read, read_across};
use crate::world::CourtroomWorld;

#[path = "model/action.rs"]
mod action;
#[path = "model/state.rs"]
mod state;

use action::{Action, BranchName, BranchPosture, EffectAction, EffectPosture};
use state::IndependentModel;

const ORDINARY_SEEDS: [u64; 3] = [0x9173, 0x51A7, 0xCAFE];
const SCHEDULED_SEEDS: [u64; 8] = [
    0x9173, 0x51A7, 0xCAFE, 0xDEAD, 0xBEEF, 0x1234, 0x5EED, 0xA11C,
];

#[derive(Default)]
struct ActionCoverage {
    creations: usize,
    selections: [bool; 4],
    reads: usize,
    relational: usize,
    signal: usize,
    combined: usize,
    retained_reads: usize,
    cleanups: usize,
}

struct ResourceBaseline {
    signal_nodes: usize,
}

pub(super) fn assert_seeded_public_action_roster() {
    for seed in ORDINARY_SEEDS {
        run(seed, 6).assert_complete(6);
    }
}

pub(super) fn assert_scheduled_public_action_roster() {
    const ROUNDS: usize = 12;
    eprintln!(
        "model_profile=scheduled seeds={} rounds_per_seed={ROUNDS} roster=fork/select/read/relational_only/signal_only/combined/retained_read/retirement/cleanup",
        SCHEDULED_SEEDS.len()
    );
    for seed in SCHEDULED_SEEDS {
        let coverage = run(seed, ROUNDS);
        coverage.assert_complete(ROUNDS);
        eprintln!(
            "model_seed_complete profile=scheduled seed=0x{seed:016x} rounds={ROUNDS} creations={} selections={} reads={} relational={} signal={} combined={} retained_reads={} cleanups={}",
            coverage.creations,
            coverage.selections.iter().filter(|seen| **seen).count(),
            coverage.reads,
            coverage.relational,
            coverage.signal,
            coverage.combined,
            coverage.retained_reads,
            coverage.cleanups,
        );
    }
}

fn run(seed: u64, rounds: usize) -> ActionCoverage {
    let world = CourtroomWorld::publish("ready");
    let root = world.application.current_world();
    let baseline = ResourceBaseline {
        signal_nodes: world.application.owned_signal_active_node_count_for_test(),
    };
    let mut branches = BTreeMap::from([(BranchName::Root, root)]);
    let mut expected = IndependentModel::new();
    let mut coverage = ActionCoverage::default();
    let mut prefix = Vec::new();

    verify_model(&world, &branches, &expected);
    for current in action::sequence(seed, rounds) {
        prefix.push(current.label());
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            execute(
                &world,
                &mut branches,
                &mut expected,
                &mut coverage,
                &baseline,
                current,
            );
            verify_model(&world, &branches, &expected);
        }));
        if let Err(failure) = outcome {
            let cause = panic_message(failure.as_ref());
            panic!(
                "seed=0x{seed:016x} failing_operation_prefix=[{}] cause={cause}",
                prefix.join(" -> ")
            );
        }
    }
    coverage
}

fn execute(
    world: &CourtroomWorld,
    branches: &mut BTreeMap<BranchName, WorthQueryProductBranch>,
    expected: &mut IndependentModel,
    coverage: &mut ActionCoverage,
    baseline: &ResourceBaseline,
    action: Action,
) {
    match action {
        Action::Fork(posture) => {
            let root = branches[&BranchName::Root];
            let branch = create_branch(world, root, posture);
            assert!(branches.insert(posture.branch(), branch).is_none());
            expected.fork_from_root(posture.branch());
            coverage.creations += 1;
        }
        Action::Select(posture) => {
            let branch = branches[&posture.branch()];
            let selected = world
                .application
                .on_branch(branch)
                .select()
                .expect("the seeded model branch must be selectable");
            assert_eq!(selected.product().product_branch(), branch);
            coverage.selections[posture.index()] = true;
        }
        Action::Read(branch) => {
            assert_observation(
                world,
                branches[&branch],
                branch,
                expected.product(branch).input(),
            );
            coverage.reads += 1;
        }
        Action::Effect(effect) => apply_modeled_effect(world, branches, expected, coverage, effect),
        Action::Retire(branch) => {
            let handle = branches[&branch];
            let close = world
                .application
                .on_branch(handle)
                .close()
                .expect("the modeled branch must close through the application facade");
            assert!(close.is_complete());
            branches.remove(&branch);
            expected.retire(branch);
            coverage.cleanups += 1;
        }
        Action::VerifyCleanup => {
            assert!(world.application.branches().pending_cleanup().is_empty());
            assert_eq!(
                world.application.owned_signal_active_node_count_for_test(),
                baseline.signal_nodes,
                "explicit cleanup must restore the sealed Signal topology inventory"
            );
        }
    }
}

fn create_branch(
    world: &CourtroomWorld,
    root: WorthQueryProductBranch,
    posture: BranchPosture,
) -> WorthQueryProductBranch {
    let source = world.application.on_branch(root).select().unwrap();
    match posture {
        BranchPosture::Reuse => reuse_exact_product(world, &source, "model-reuse"),
        BranchPosture::Relational => {
            fork_relational_product(world, &source, "model-r", "model-r-data")
        }
        BranchPosture::Signal => world
            .application
            .branches()
            .fork(root)
            .components(|components| components.reuse_exact_relational_basis().fork_signal())
            .create()
            .expect("the Signal-only posture must publish"),
        BranchPosture::Independent => {
            fork_independent_product(world, &source, "model-i", "model-i-data", "model-i-signal")
        }
    }
}

fn apply_modeled_effect(
    world: &CourtroomWorld,
    branches: &BTreeMap<BranchName, WorthQueryProductBranch>,
    expected: &mut IndependentModel,
    coverage: &mut ActionCoverage,
    effect: EffectAction,
) {
    let EffectAction {
        target,
        observer,
        posture,
        input,
        ordinal,
    } = effect;
    let target_branch = branches[&target];
    let observer_branch = branches[&observer];
    let target_before = read(world, target_branch);
    assert_ordinary_work(target_before.work);
    let observer_before = read(world, observer_branch);
    assert_ordinary_work(observer_before.work);
    let (retained, ()) = read_across(world, observer_branch, || {
        apply_effect(world, target_branch, &input, ordinal, posture, coverage)
    });
    expected.apply_effect(target, posture, &input);

    let target_after = read(world, target_branch);
    assert_ordinary_work(target_after.work);
    assert_ne!(
        target_after.identity.selected_commit(),
        target_before.identity.selected_commit(),
        "the target branch must advance for every effect posture"
    );
    assert_eq!(target_after.input, expected.product(target).input());
    assert_retained_work(retained.work);
    assert_eq!(retained.input, expected.product(observer).input());
    assert_eq!(retained.identity, observer_before.identity);
    coverage.retained_reads += 1;
}

fn apply_effect(
    world: &CourtroomWorld,
    branch: WorthQueryProductBranch,
    input: &str,
    ordinal: u8,
    posture: EffectPosture,
    coverage: &mut ActionCoverage,
) {
    match posture {
        EffectPosture::Relational => {
            coverage.relational += 1;
            let receipt = world
                .change_input_on_branch_with_ordinal(branch, input, ordinal)
                .require_committed()
                .expect("the modeled Relational-only change must commit");
            assert_eq!(receipt.product_branch(), branch);
        }
        EffectPosture::Signal => {
            coverage.signal += 1;
            for _ in 0..2 {
                let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
                let selected = world.application.on_branch(branch).select().unwrap();
                let cancellation = runtime::RuntimeWorldCancellationSource::new();
                let outcome = selected
                    .publish_conditional_definition(
                        &world.clock,
                        Arc::new(replacement),
                        &cancellation.token(),
                    )
                    .expect("the modeled Signal-only definition must reach World");
                let primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(
                    publication,
                ) = outcome
                else {
                    panic!("the modeled Signal-only definition must publish")
                };
                assert_eq!(publication.product_branch(), branch);
            }
        }
        EffectPosture::Combined => {
            coverage.combined += 1;
            let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
            let receipt = world.change_input_and_conditional_definition_on_branch(
                branch,
                input,
                Arc::new(replacement),
                ordinal,
            );
            assert_eq!(receipt.product_branch(), branch);
        }
    }
}

fn verify_model(
    world: &CourtroomWorld,
    branches: &BTreeMap<BranchName, WorthQueryProductBranch>,
    expected: &IndependentModel,
) {
    assert_eq!(branches.len(), expected.products().count());
    assert_eq!(
        world
            .application
            .installed_conditional_definition_count_for_test(),
        expected.installed_definition_count(),
        "the World definition inventory must equal the independent action model"
    );
    assert_eq!(
        world
            .application
            .installed_conditional_target_reference_count_for_test(),
        expected.installed_definition_count(),
        "each modeled definition must retain one exact target reference"
    );
    for (name, product) in expected.products() {
        assert_observation(world, branches[&name], name, product.input());
    }
}

fn assert_observation(
    world: &CourtroomWorld,
    branch: WorthQueryProductBranch,
    name: BranchName,
    input: &str,
) {
    let observed = read(world, branch);
    assert_ordinary_work(observed.work);
    assert_eq!(observed.input, input, "branch={}", name.label());
    assert_eq!(observed.identity.product_branch(), branch);
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic")
}

impl ActionCoverage {
    fn assert_complete(&self, rounds: usize) {
        assert_eq!(self.creations, BranchPosture::ALL.len());
        assert!(self.selections.into_iter().all(|seen| seen));
        assert_eq!(self.reads, BranchName::NON_ROOT.len());
        assert!(self.relational > 0);
        assert!(self.signal > 0);
        assert!(self.combined > 0);
        assert_eq!(self.retained_reads, rounds);
        assert_eq!(self.cleanups, BranchName::NON_ROOT.len());
    }
}

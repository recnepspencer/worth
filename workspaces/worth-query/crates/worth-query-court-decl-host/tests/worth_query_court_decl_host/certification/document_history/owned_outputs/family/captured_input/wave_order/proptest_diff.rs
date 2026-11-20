//! Fixed-seed 64-case `proptest` base-order differential for PR B (#254).
//!
//! Covers ranks, captured readers, supplier-field writers, outgoing selectors,
//! multi-stage `allowed` barriers, alias coalescing with static continuation
//! tails, `refresh_input_captures` mid-batch breaks, and `Recapture` cuts.
use super::super::super::super::synchronous_edit::{
    envelope_fixture as fixture, interleaved::component_carriage as carriage,
};
use super::super::*;
use super::{compare_or_record, install, r3_shapes, summarize_outcome, Trace};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

const GOLDEN: &str = "tests/worth_query_court_decl_host/certification/document_history/owned_outputs/family/captured_input/wave_order_proptest.golden";
const SEED: [u8; 32] = *b"worth-pr-b-wave-order-oracle-v01";

#[derive(Clone, Debug)]
enum CarriageStep {
    Model(u8, u64),
    After(u64),
    Member(u64),
    ObserveChild,
    AttachLate,
}
#[derive(Clone, Debug)]
enum Scenario {
    /// Multi-stage barriers + alias coalescing with static continuation tails.
    Carriage {
        pre_seed: bool,
        steps: Vec<CarriageStep>,
    },
    /// `refresh_input_captures` mid-batch breaks (with or without retained parent).
    InputCaptureSwitch { retain_old: bool, value: u64 },
    /// Same-rank `PlannedOutput::Recapture` mid-batch cut (healthy vs changing).
    RecaptureSplit { second_value: u64 },
    /// Composed ranks, captured readers, supplier writers, waits, outgoing selectors.
    Composed { waits: bool, second_value: u64 },
    /// Width × length chain levels.
    Chains {
        width: usize,
        length: usize,
        second_value: u64,
    },
    /// Supplier-driven wide captured readers (unchanged vs changing).
    WideSupplier { width: usize, second_value: u64 },
}

fn carriage_step() -> impl Strategy<Value = CarriageStep> {
    prop_oneof![
        3 => (0..2u8, 2..6u64).prop_map(|(k, v)| CarriageStep::Model(k, v)),
        2 => (1..4u64).prop_map(CarriageStep::After),
        2 => (1..4u64).prop_map(CarriageStep::Member),
        1 => Just(CarriageStep::ObserveChild),
        1 => Just(CarriageStep::AttachLate),
    ]
}
fn scenario() -> impl Strategy<Value = Scenario> {
    prop_oneof![
        4 => (any::<bool>(), prop::collection::vec(carriage_step(), 1..=4))
            .prop_map(|(pre_seed, steps)| Scenario::Carriage { pre_seed, steps }),
        2 => (any::<bool>(), 3..6u64)
            .prop_map(|(retain_old, value)| Scenario::InputCaptureSwitch { retain_old, value }),
        2 => (4..6u64).prop_map(|second_value| Scenario::RecaptureSplit { second_value }),
        2 => (any::<bool>(), 4..6u64)
            .prop_map(|(waits, second_value)| Scenario::Composed { waits, second_value }),
        2 => (1..=3usize, 1..=3usize, 4..6u64)
            .prop_map(|(width, length, second_value)| Scenario::Chains { width, length, second_value }),
        2 => (1..=3usize, 4..6u64)
            .prop_map(|(width, second_value)| Scenario::WideSupplier { width, second_value }),
    ]
}

fn run_carriage(trace: &mut Trace, pre_seed: bool, steps: &[CarriageStep]) {
    type Host = WorthQueryProgramApplicationRuntime<Schema, carriage::Program>;
    let host: Host = install(world::Configuration {
        interleaved_records: true,
        ..Default::default()
    });
    if pre_seed {
        let _ = edit_stage!(&host, carriage::Tree, "seed", "a", 2);
    }
    let obs = WorthQueryExecutionObservation::install_on_this_thread();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("proptest-carriage".into())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    let mut attached = false;
    for step in steps {
        match step {
            CarriageStep::Model(0, value) => {
                edit.push_stage::<carriage::Tree, _>(fixture::model::Intent {
                    key: "carriage:b".into(),
                    value: *value,
                    key_derived_target: false,
                })
                .unwrap();
            }
            CarriageStep::Model(_, value) => {
                edit.push_stage::<carriage::Tree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: *value,
                    key_derived_target: false,
                })
                .unwrap();
            }
            CarriageStep::After(value) => {
                edit.push_stage::<carriage::AfterTree, _>(fixture::after_producer::Intent {
                    key: "envelope-cap".into(),
                    value: *value,
                    key_derived_target: false,
                })
                .unwrap();
            }
            CarriageStep::Member(value) => {
                edit.push_stage::<carriage::MemberTree, _>(fixture::member::Intent {
                    key: "m".into(),
                    value: *value,
                    key_derived_target: false,
                })
                .unwrap();
            }
            CarriageStep::ObserveChild => {
                edit.push_stage::<ApplicationNoOutputGraph, _>(
                    carriage::value_observation::Intent {
                        model: "a".into(),
                        source: carriage::value_observation::ValueSource::Child(
                            "from-first:first:a".into(),
                        ),
                    },
                )
                .unwrap();
            }
            CarriageStep::AttachLate if !attached => {
                attached = true;
                edit.push_stage::<ApplicationNoOutputGraph, _>(carriage::attach::Intent {
                    parent: "a".into(),
                    child: "carriage:late".into(),
                    pending_child: false,
                })
                .unwrap();
            }
            CarriageStep::AttachLate => {}
        }
    }
    let result = match edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation) {
        Ok(outcome) => summarize_outcome(outcome, obs),
        Err(denial) => super::summarize_denial(&denial, obs),
    };
    trace.edit("run", result);
    trace.rows(
        &host,
        &["first:a", "first:carriage:b", "first:carriage:late"],
    );
}

fn run_scenario(trace: &mut Trace, item: &Scenario) {
    match item {
        Scenario::Carriage { pre_seed, steps } => run_carriage(trace, *pre_seed, steps),
        Scenario::InputCaptureSwitch { retain_old, value } => {
            type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
            let host: Host = install(world::Configuration {
                interleaved_records: true,
                captured_inputs: true,
                captured_parent_switch: Some(*retain_old),
                ..Default::default()
            });
            trace.edit(
                "switch",
                edit_stage!(&host, program::Tree, "switch", "capture:child", *value),
            );
            trace.rows(&host, &["captured:capture:child", "first:carriage:d"]);
        }
        Scenario::RecaptureSplit { second_value } => {
            let host = r3_shapes::batch_split::build_host();
            let _ = edit_stage!(
                &host,
                r3_shapes::batch_split::Tree,
                "split-born",
                "carriage:d",
                4
            );
            trace.edit(
                "second",
                edit_stage!(
                    &host,
                    r3_shapes::batch_split::Tree,
                    "split-second",
                    "carriage:d",
                    *second_value
                ),
            );
            trace.rows(&host, &["captured:capture:x", "captured:capture:z"]);
        }
        Scenario::Composed {
            waits,
            second_value,
        } => {
            type Host = WorthQueryProgramApplicationRuntime<Schema, composed::Program>;
            let host: Host = install(world::Configuration {
                captured_composed: Some((0, *waits)),
                ..Default::default()
            });
            let _ = edit_stage!(&host, composed::Tree, "born", "carriage:d", 4);
            trace.edit(
                "second",
                edit_stage!(&host, composed::Tree, "second", "carriage:d", *second_value),
            );
            trace.rows(&host, &["captured:capture:p", "captured:capture:r"]);
        }
        Scenario::Chains {
            width,
            length,
            second_value,
        } => {
            type Host = WorthQueryProgramApplicationRuntime<Schema, chains::Program>;
            let host: Host = install(world::Configuration {
                captured_chains: Some((*width, *length)),
                ..Default::default()
            });
            let _ = edit_stage!(&host, chains::Tree, "born", "carriage:d", 4);
            trace.edit(
                "second",
                edit_stage!(&host, chains::Tree, "second", "carriage:d", *second_value),
            );
            let last = format!("first:{}", chains::key(width - 1, length - 1));
            trace.rows(&host, &["first:chain:0:0", &last]);
        }
        Scenario::WideSupplier {
            width,
            second_value,
        } => {
            type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
            let host: Host = install(world::Configuration {
                captured_composed: Some((*width, false)),
                ..Default::default()
            });
            let _ = edit_stage!(&host, supplier::SupplierTree, "born", "carriage:d", 4);
            trace.edit(
                "second",
                edit_stage!(
                    &host,
                    supplier::SupplierTree,
                    "second",
                    "carriage:d",
                    *second_value
                ),
            );
            trace.rows(&host, &["captured:capture:w0", "first:carriage:d"]);
        }
    }
}

#[test]
fn fixed_seed_proptest_preserves_base_order_across_64_cases() {
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: 64,
            failure_persistence: None,
            ..Config::default()
        },
        TestRng::from_seed(RngAlgorithm::ChaCha, &SEED),
    );
    let strategy = scenario();
    let mut trace = Trace(String::new());
    let mut saw_input_capture_break = false;
    let mut saw_recapture_break = false;
    let mut saw_carriage = false;
    for case in 0..64usize {
        let value = strategy.new_tree(&mut runner).unwrap().current();
        if matches!(value, Scenario::Carriage { .. }) {
            saw_carriage = true;
        }
        trace.0 += &format!("== case {case:02} {value:?}\n");
        let start = trace.0.len();
        run_scenario(&mut trace, &value);
        let slice = &trace.0[start..];
        saw_input_capture_break |= slice.contains("break input-captures");
        saw_recapture_break |= slice.contains("break recapture");
    }
    assert!(saw_carriage, "generator must exercise multi-stage carriage");
    assert!(
        saw_input_capture_break,
        "generator must exercise refresh_input_captures mid-batch breaks"
    );
    assert!(
        saw_recapture_break,
        "generator must exercise Recapture mid-batch breaks"
    );
    compare_or_record(GOLDEN, &trace.0);
}

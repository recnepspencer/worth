use worth_ui::facade::app::WorthUiActiveApplicationSession;
use worth_ui::facade::declaration::ComponentId;
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::UiRebindExecutionPolicy;
use worth_ui::facade::source::WorthUiFilesystemSourceProvider;
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_test_support::WorthUiActiveSessionCertificationExt;

use super::filesystem_contract_workspace::FilesystemContractWorkspace;

const INFLATED_UNRELATED_WIDTH: usize = 32;

#[derive(Debug, Eq, PartialEq)]
struct PostClassificationCostVector {
    observations: usize,
    changed_facts: usize,
    affected_aspects: usize,
    indexed_consumers: usize,
    selected_decisions: usize,
    graph_and_mounted_entries: usize,
    measurement_and_allocation_entries: usize,
    binding_transitions: usize,
}

#[derive(Debug)]
struct CostWorldReceipt {
    graph_width: usize,
    cost: PostClassificationCostVector,
    lookup_receipts: usize,
    index_probes: usize,
    contract_checks: usize,
    planned_effects: usize,
}

#[test]
fn rebind_post_classification_cost_is_independent_of_unrelated_width() {
    prove_rebind_post_classification_cost_is_independent_of_unrelated_width();
}

pub(crate) fn prove_rebind_post_classification_cost_is_independent_of_unrelated_width() {
    let baseline = compile_platform_pulse_plan("phase-312-cost-baseline", 0);
    let inflated = compile_platform_pulse_plan("phase-312-cost-inflated", INFLATED_UNRELATED_WIDTH);

    assert_eq!(baseline.graph_width, 4);
    assert_eq!(
        inflated.graph_width,
        baseline.graph_width + INFLATED_UNRELATED_WIDTH,
        "the independent graph authority must prove that the second world is wider"
    );
    assert_eq!(inflated.cost, baseline.cost);
    assert_eq!(inflated.lookup_receipts, baseline.lookup_receipts);
    assert_eq!(inflated.index_probes, baseline.index_probes);
    assert_eq!(inflated.contract_checks, baseline.contract_checks);
    assert_eq!(inflated.planned_effects, baseline.planned_effects);

    assert_eq!(
        baseline.cost,
        PostClassificationCostVector {
            observations: 1,
            changed_facts: 1,
            affected_aspects: 0,
            indexed_consumers: 2,
            selected_decisions: 2,
            graph_and_mounted_entries: 0,
            measurement_and_allocation_entries: 0,
            binding_transitions: 0,
        }
    );
    assert_eq!(baseline.lookup_receipts, 2);
    assert_eq!(baseline.index_probes, 2);
    assert_eq!(baseline.contract_checks, 4);
    assert_eq!(baseline.planned_effects, 1);
}

fn compile_platform_pulse_plan(label: &str, unrelated_width: usize) -> CostWorldReceipt {
    let scenario = FilesystemApplicationLifecycleScenario::new(label);
    let workspace = FilesystemContractWorkspace::new(label);
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::platform_pulse_source_text_with_unrelated_width(
            unrelated_width,
        ),
    );
    let initial = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("scaled initial filesystem world reads");
    let capabilities = scenario.platform_pulse_capability_application_with_unrelated_width(
        WorthUiHeadlessRecorder::default(),
        unrelated_width,
    );
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        initial,
        capabilities.capabilities(),
    );
    let mut session = scenario
        .prepare_platform_pulse_application_with_unrelated_width(
            submission,
            WorthUiHeadlessRecorder::default(),
            unrelated_width,
        )
        .launch()
        .expect("scaled Platform Pulse world launches");
    let receipt = compile_candidate_plan(&mut session, &workspace, unrelated_width);
    let _ = session.shutdown();
    workspace.close();
    receipt
}

fn compile_candidate_plan(
    session: &mut WorthUiActiveApplicationSession,
    workspace: &FilesystemContractWorkspace,
    unrelated_width: usize,
) -> CostWorldReceipt {
    let graph = session.graph();
    let graph_width = graph.node_count();
    let graph_declarations = graph
        .node_identities()
        .map(|identity| {
            graph
                .inspection()
                .inspect_graph_node(identity)
                .expect("enumerated graph node remains inspectable")
                .value()
                .declaration_identity()
                .authored_semantic_name()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert!(
        graph_declarations
            .iter()
            .any(|identity| identity == "component:platform.pulse.component.seed"),
        "the real source-backed graph must retain the Pulse background declaration: {graph_declarations:?}"
    );
    let background = session
        .capabilities()
        .components()
        .get(&ComponentId::new("platform.pulse.component.seed").unwrap())
        .expect("real Pulse predecessor retains its background component capability");
    assert_eq!(background.surface_paint_order(), Some(0));
    workspace.write(
        "app/main.wui",
        &FilesystemApplicationLifecycleScenario::platform_pulse_green_source_text_with_unrelated_width(
            unrelated_width,
        ),
    );
    let candidate = WorthUiFilesystemSourceProvider::new(workspace.root())
        .read()
        .expect("scaled candidate filesystem world reads");
    let candidate =
        FilesystemApplicationLifecycleScenario::lower_snapshot(candidate, session.capabilities());
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    let changed = match session.classify_observations(admitted).unwrap() {
        UiChangeClassificationOutcome::Changed(changed) => changed,
        _ => panic!("the blue-to-green edit must remain semantically changed"),
    };
    let token_receipt = session
        .lookup_consumed_fact(&changed.facts()[0])
        .expect("the predecessor index must resolve the classified token");
    assert_eq!(token_receipt.entries().len(), 2);
    let scope = session
        .resolve_affected_scope(changed)
        .expect("scaled affected scope remains within the Platform Pulse profile");
    let scope_cost = scope.cost();
    let lifecycle = scope
        .resolve_identity_lifecycle()
        .expect("scaled identity lifecycle resolves");
    let plan = session
        .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
        .expect("scaled rebind plan compiles");
    let plan_cost = plan.cost();
    let cost = PostClassificationCostVector {
        observations: scope_cost.observations(),
        changed_facts: scope_cost.changed_facts(),
        affected_aspects: scope_cost.affected_aspects(),
        indexed_consumers: scope_cost.indexed_consumers(),
        selected_decisions: plan_cost.selected_decisions(),
        graph_and_mounted_entries: plan_cost.graph_and_mounted_entries(),
        measurement_and_allocation_entries: plan_cost.measurement_and_allocation_entries(),
        binding_transitions: plan_cost.binding_transitions(),
    };
    CostWorldReceipt {
        graph_width,
        cost,
        lookup_receipts: scope_cost.lookup_receipts(),
        index_probes: scope_cost.index_probes(),
        contract_checks: scope_cost.contract_checks(),
        planned_effects: plan_cost.effects(),
    }
}

use super::super::SignalBranchBasisPort;
use crate::branch::{
    AdmittedSignalBranchBasis, RecordedSignalInitialEmptyRoot, SignalInitialEmptyRootRefusal,
};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;

fn seal(
    graph: SignalGraph,
) -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalBranchBasisPort<(), (), ()>,
    AdmittedSignalBranchBasis,
) {
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (port, _, _) = runtime.owner_port_slots().unwrap();
    (runtime, port, basis)
}

#[test]
fn recorded_initial_empty_root_readmits_through_a_new_actual_owner() {
    let (runtime, port, basis) = seal(SignalGraph::new());
    let recorded = port.export_recorded_initial_empty_root(&basis).unwrap();
    let encoded = serde_json::to_vec(&recorded).unwrap();
    drop(basis);
    drop(runtime);
    assert!(matches!(
        port.readmit_recorded_initial_empty_root(&recorded),
        Err(SignalInitialEmptyRootRefusal::OwnerUnavailable)
    ));

    let (_new_runtime, new_port, new_basis) = seal(SignalGraph::new());
    let decoded = serde_json::from_slice(&encoded).unwrap();
    let admitted = new_port
        .readmit_recorded_initial_empty_root(&decoded)
        .unwrap();
    assert_eq!(admitted.basis().descriptor(), new_basis.descriptor());
    new_port.compare_current_exact(admitted.basis()).unwrap();
}

#[test]
fn initial_empty_root_requires_untouched_producer_origin_and_exact_meaning() {
    let mut modified = SignalGraph::new();
    let node = modified.node().build();
    modified.rollback_created_nodes(&[node]);
    let empty_wire = serde_json::to_vec(&SignalGraph::new()).unwrap();
    for graph in [
        modified,
        SignalGraph::new().clone(),
        serde_json::from_slice(&empty_wire).unwrap(),
    ] {
        let (_runtime, port, basis) = seal(graph);
        assert!(matches!(
            port.export_recorded_initial_empty_root(&basis),
            Err(SignalInitialEmptyRootRefusal::InitialRootRequired)
        ));
    }

    let (_runtime, port, basis) = seal(SignalGraph::new());
    let recorded = port.export_recorded_initial_empty_root(&basis).unwrap();
    let (_foreign_runtime, foreign, foreign_basis) = seal(SignalGraph::new());
    assert!(matches!(
        port.export_recorded_initial_empty_root(&foreign_basis),
        Err(SignalInitialEmptyRootRefusal::CurrentBasisRequired)
    ));
    let mut changed = serde_json::to_value(&recorded).unwrap();
    changed["schema_commitment"] = "foreign schema".into();
    let changed: RecordedSignalInitialEmptyRoot = serde_json::from_value(changed).unwrap();
    assert!(matches!(
        foreign.readmit_recorded_initial_empty_root(&changed),
        Err(SignalInitialEmptyRootRefusal::RecordedMeaningMismatch)
    ));
    super::world::advance_exact(&port, &basis);
    assert!(matches!(
        port.readmit_recorded_initial_empty_root(&recorded),
        Err(SignalInitialEmptyRootRefusal::CurrentBasisRequired)
    ));
}

use worth_proof::TransitionOutcome;
use worth_signal::facade::{
    apply_installed_scoped_changes, Aspect, ChangedRegion, InstalledSignalScopedChange,
    ScopeCoverage, ScopePath, SignalGraph,
};

#[test]
fn wasm_supported_facade_retains_full_depth_opaque_locality() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let aspect = Aspect::new(0);
    let TransitionOutcome::Success(capability) = graph.admit_installed_aspect(source, aspect)
    else {
        panic!("the WASM-supported Signal facade must admit its own installed aspect")
    };
    let segments: Vec<_> = (0..8)
        .map(|level| format!("opaque:segment:{level}"))
        .collect();
    let TransitionOutcome::Success(admitted) = apply_installed_scoped_changes(
        &mut graph,
        [InstalledSignalScopedChange::new(
            capability,
            [ChangedRegion::exact(
                ScopePath::new(segments.clone()).unwrap(),
            )],
        )],
    ) else {
        panic!("the WASM-supported facade must admit exact scoped invalidation")
    };

    let change = admitted.changes().next().unwrap();
    assert_eq!(change.aspect(), aspect);
    assert_eq!(change.changed_regions()[0].path().segments(), segments);
    assert_eq!(change.changed_regions()[0].coverage(), ScopeCoverage::Exact);
}

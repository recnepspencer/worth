use worth_proof::TransitionOutcome;

use crate::data::graph::SignalGraph;
use crate::data::node::NodeState;
use crate::data::output::ChangedRegion;
use crate::data::output::PartitionSubscription;

use super::*;
use crate::data::aspect::Aspect;

#[test]
fn installed_scoped_change_retains_aspect_correlated_regions() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let aspect_a = Aspect::new(0);
    let aspect_b = Aspect::new(1);
    let TransitionOutcome::Success(a) = graph.admit_installed_aspect(source, aspect_a) else {
        panic!("source aspect A must admit")
    };
    let TransitionOutcome::Success(b) = graph.admit_installed_aspect(source, aspect_b) else {
        panic!("source aspect B must admit")
    };

    let TransitionOutcome::Success(admitted) = apply_installed_scoped_changes(
        &mut graph,
        [
            InstalledSignalScopedChange::new(a, [ChangedRegion::new("rates").with_detail("5y")]),
            InstalledSignalScopedChange::new(b, [ChangedRegion::new("credit").with_detail("ig")]),
        ],
    ) else {
        panic!("current installed scoped changes must admit")
    };

    let changes = admitted.changes().collect::<Vec<_>>();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].aspect(), aspect_a);
    assert_eq!(
        changes[0].changed_regions()[0].path().segments(),
        ["rates", "5y"]
    );
    assert_eq!(changes[1].aspect(), aspect_b);
    assert_eq!(
        changes[1].changed_regions()[0].path().segments(),
        ["credit", "ig"]
    );
    assert_eq!(graph.get_state(source).unwrap(), NodeState::Dirty);
    let rates_5y = PartitionSubscription::partition_and_detail("rates", "5y");
    let credit_ig = PartitionSubscription::partition_and_detail("credit", "ig");
    assert_eq!(
        graph
            .node_version_for_scope(source, aspect_a, Some(&rates_5y))
            .unwrap(),
        1
    );
    assert_eq!(
        graph
            .node_version_for_scope(source, aspect_b, Some(&rates_5y))
            .unwrap(),
        0
    );
    assert_eq!(
        graph
            .node_version_for_scope(source, aspect_a, Some(&credit_ig))
            .unwrap(),
        0
    );
    assert_eq!(
        graph
            .node_version_for_scope(source, aspect_b, Some(&credit_ig))
            .unwrap(),
        1
    );
}

#[test]
fn duplicate_and_foreign_capabilities_fail_before_effects() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let aspect = Aspect::new(0);
    let TransitionOutcome::Success(first) = graph.admit_installed_aspect(source, aspect) else {
        panic!("first capability must admit")
    };
    let TransitionOutcome::Success(duplicate) = graph.admit_installed_aspect(source, aspect) else {
        panic!("duplicate capability must be constructible for denial proof")
    };
    let before = graph.node_aspect_version(source).unwrap();
    let duplicate = apply_installed_scoped_changes(
        &mut graph,
        [
            InstalledSignalScopedChange::new(first, []),
            InstalledSignalScopedChange::new(duplicate, []),
        ],
    );
    assert!(matches!(
        duplicate,
        TransitionOutcome::Denied(SignalInstalledScopedChangeDenial::DuplicateTarget)
    ));
    assert_eq!(graph.node_aspect_version(source).unwrap(), before);

    let mut foreign_graph = SignalGraph::new();
    let foreign_source = foreign_graph.node().build();
    let TransitionOutcome::Success(foreign) =
        foreign_graph.admit_installed_aspect(foreign_source, aspect)
    else {
        panic!("foreign capability must admit in its owner graph")
    };
    let foreign =
        apply_installed_scoped_changes(&mut graph, [InstalledSignalScopedChange::new(foreign, [])]);
    assert!(matches!(
        foreign,
        TransitionOutcome::Denied(SignalInstalledScopedChangeDenial::ForeignCapability)
    ));
    assert_eq!(graph.node_aspect_version(source).unwrap(), before);
}

#[test]
fn invalid_scope_path_is_rejected_before_scoped_batch_construction() {
    assert_eq!(
        crate::data::output::ScopePath::one(""),
        Err(crate::data::output::ScopePathError::EmptySegment)
    );
}

//! The ordinary authored ring journey across native fork and checkpoint owners.
use super::*;

#[test]
fn authored_ring_fork_diverges_and_parent_remains_current() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let parent = Court::new(&application, &request, 0x9176_3e80);
    let mut rings = vec![Ring::seeded(0), Ring::created(1)];
    parent.create_ring(1, "ordinary authoring");
    Reading::decisions();
    for index in 0..rings.len() {
        parent.demand_ring(&mut rings, index, "cold publication");
    }
    let branch = support::host_call(&application, |phase| {
        application
            .branches()
            .fork(application.current_world())
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(phase)
            .map_err(Box::new)
    })
    .unwrap();
    let child_request = application.request(&principal, &scope).on_branch(branch);
    let child = Court::new(&application, &child_request, 0x9176_3ec0);
    let mut child_rings = rings.clone();
    let (fork_cost, _) = child.demand_ring(&mut child_rings, 1, "native fork admission");
    eprintln!("native fork admission baseline: {fork_cost:?}");

    let before = root_contacts(&child, &child_rings);
    child_rings[1].a_y = 6;
    child.write_y(&child_rings[1].key("a"), 6, "child source edit");
    child.demand_ring(&mut child_rings, 1, "child publication");
    assert_eq!(
        affected_roots(&child, &child_rings, &before),
        [child_rings[1].key("a"), child_rings[1].key("b")],
        "the child's exact producer set includes no other ring or downstream cutoff"
    );
    for index in 0..rings.len() {
        let (costs, decisions) = parent.demand_ring(&mut rings, index, "parent after divergence");
        assert!(costs.iter().all(Cost::is_free));
        assert_eq!(decisions, 0);
    }
    assert_eq!(parent.length(&rings[1].key("a")), 2);
    assert_eq!(child.length(&child_rings[1].key("a")), 7);
}

#[test]
fn authored_ring_checkpoint_reopens_then_accepts_a_new_source_edit() {
    let _guard = checkpoint_recovery_test_guard();
    let mut rings = vec![Ring::seeded(0), Ring::created(1)];
    Reading::decisions();
    let checkpoint = {
        let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let court = Court::new(&application, &request, 0x9176_3f80);
        court.create_ring(1, "ordinary authoring");
        court.write_y(&rings[1].key("a"), 6, "authored value before capture");
        rings[1].a_y = 6;
        for index in 0..rings.len() {
            court.demand_ring(&mut rings, index, "publication before capture");
        }
        application
            .with_host_advancement(|phase| {
                application.capture_application_checkpoint(
                    &phase,
                    worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
                ).unwrap()
            })
            .expect("the native capture call is admitted")
    };
    let restored = install(
        Some(checkpoint),
        |_| panic!("native reopen must preserve authored truth without reseeding"),
        Retained::AMPLE,
    );
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let court = Court::new(&restored, &request, 0x9176_4000);
    for index in 0..rings.len() {
        let (costs, _) = court.demand_ring(&mut rings, index, "native checkpoint readmission");
        eprintln!("checkpoint readmission ring {index}: {costs:?}");
        let (warm, decisions) = court.demand_ring(&mut rings, index, "continuous after reopen");
        assert!(warm.iter().all(Cost::is_free));
        assert_eq!(decisions, 0);
    }
    // Equal authored input has no new semantic source value to recompute.
    court.write_y(&rings[1].key("a"), 6, "equal source value");
    let (equal, decisions) = court.demand_ring(&mut rings, 1, "equal input demand");
    assert!(equal.iter().all(Cost::is_free));
    assert_eq!(decisions, 0);

    let before = root_contacts(&court, &rings);
    rings[1].a_y = 7;
    court.write_y(&rings[1].key("a"), 7, "new source value after reopen");
    court.demand_ring(&mut rings, 1, "new publication after reopen");
    assert_eq!(
        affected_roots(&court, &rings, &before),
        [rings[1].key("a"), rings[1].key("b")]
    );
    let (untouched, decisions) = court.demand_ring(&mut rings, 0, "unrelated restored ring");
    assert!(untouched.iter().all(Cost::is_free));
    assert_eq!(decisions, 0);
    court.judge_chain(&rings[1], "new published result");
}

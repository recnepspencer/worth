use super::*;
use crate::runtime::intent::operability::UiIntentOperabilityDecision;
use crate::runtime::persistent_index::{begin_all_test_observation, test_work};

// These tests exercise private storage and its cost. The mounted integration
// test derives standing facts through real pointer ingress and Intent evaluation.
fn row(
    instance: UiMountedInstanceIdentity,
    binding: UiSurfaceBindingGeneration,
    routes: usize,
    revision: u64,
) -> InstanceFacts {
    let mut facts = UiPersistentOrdMap::default();
    for route in 0..routes {
        let name = format!("route.{route}");
        facts.insert(
            name.clone().into_boxed_str(),
            UiIntentOperabilityStandingFact::for_test(
                crate::graph::UiGraphNodeIdentity::new(71),
                instance,
                worth_ui_host_contract::UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
                &name,
                revision,
            ),
        );
    }
    InstanceFacts {
        binding,
        routes: facts,
    }
}

#[test]
fn operability_snapshot_lookup_and_indexed_retirement_preserve_unrelated_instances() {
    // Four is the current canonical interaction-family count. Route comparison
    // is bounded per affected instance, not independent of its route count.
    for (size, routes) in [(64usize, 1), (4_096, 4)] {
        let mut owner = UiIntentOperabilityStandingOwner::default();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let peer_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let mut instances = Vec::new();
        for index in 0..size {
            let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
            owner.replace_instance(
                instance,
                row(
                    instance,
                    if index < 2 { binding } else { peer_binding },
                    routes,
                    1,
                ),
            );
            instances.push(instance);
        }
        owner.revision = 1;
        begin_all_test_observation();
        let before = owner.snapshot();
        assert_eq!(before, before.clone());
        assert_eq!(test_work().lookup_probes(), 0);
        let graph = crate::graph::UiGraphNodeIdentity::new(71);
        assert!(before
            .fact_for(graph, instances[size / 2], "route.0")
            .is_some());
        assert!(before
            .fact_for(
                crate::graph::UiGraphNodeIdentity::new(72),
                instances[size / 2],
                "route.0"
            )
            .is_none());
        assert!(before
            .fact_for(graph, instances[size / 2], "missing")
            .is_none());
        assert_eq!(test_work().iterated_entries(), 0);
        assert!(test_work().lookup_probes() <= 6 * size.ilog2() as usize);

        begin_all_test_observation();
        owner.retire_binding(binding);
        assert_eq!(
            test_work().iterated_entries(),
            2 + 2 * routes,
            "retirement visits the binding's membership and each member's routes"
        );
        let retired = owner.snapshot();
        assert_eq!(retired.changed_instances(&before).as_ref(), &instances[..2]);
        assert!(before.fact_for(graph, instances[0], "route.0").is_some());
        assert!(retired.fact_for(graph, instances[0], "route.0").is_none());
        assert!(retired.fact_for(graph, instances[2], "route.0").is_some());
        owner.retire_binding(binding);
        assert_eq!(owner.snapshot(), retired);

        owner.retire_instance(instances[2]);
        let single = owner.snapshot();
        assert_eq!(single.changed_instances(&retired).as_ref(), &[instances[2]]);
        assert!(
            !owner
                .bindings
                .get(&peer_binding)
                .unwrap()
                .contains_with_probes(&instances[2])
                .0
        );
        owner.clear();
        assert!(owner.snapshot().facts.is_empty());
        assert!(owner.bindings.is_empty());
        assert_eq!(before.facts.len(), size);
    }
}

#[test]
fn operability_binding_replacement_and_exhaustion_preserve_index_agreement() {
    let mut owner = UiIntentOperabilityStandingOwner::default();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let old = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let new = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    owner.replace_instance(instance, row(instance, old, 1, 1));
    owner.replace_instance(instance, row(instance, new, 1, 2));
    assert!(owner.bindings.get(&old).is_none());
    assert!(
        owner
            .bindings
            .get(&new)
            .unwrap()
            .contains_with_probes(&instance)
            .0
    );
    owner.retire_binding(old);
    assert_eq!(owner.facts.len(), 1);
    owner.revision = u64::MAX;
    let before = owner.snapshot();
    let failed =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| owner.retire_binding(new)));
    assert!(failed.is_err());
    assert_eq!(owner.snapshot(), before);
    assert!(
        owner
            .bindings
            .get(&new)
            .unwrap()
            .contains_with_probes(&instance)
            .0
    );
}

#[test]
fn physical_surface_rebind_preserves_standing_meaning_under_the_successor_binding() {
    let mut owner = UiIntentOperabilityStandingOwner::default();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let predecessor = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let successor = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    owner.replace_instance(instance, row(instance, predecessor, 2, 7));
    let before = owner.snapshot();

    owner.rebind_surface(predecessor, successor);

    assert_eq!(owner.revision, before.owner_revision());
    assert!(owner.bindings.get(&predecessor).is_none());
    assert!(
        owner
            .bindings
            .get(&successor)
            .unwrap()
            .contains_with_probes(&instance)
            .0
    );
    assert_eq!(owner.facts.get(&instance).unwrap().binding, successor);
    assert_eq!(owner.snapshot().facts().len(), 2);
}

fn readonly() -> UiIntentOperabilityDecision {
    use crate::runtime::intent::operability::*;
    UiIntentOperabilityDecision::new(UiIntentOperabilityDecisionInput {
        contract_identity: "appearance-state-test-intent".into(),
        support: UiIntentSupportPosture::Supported,
        mutability: UiIntentMutabilityPosture::Readonly,
        readiness: UiIntentReadinessPosture::Ready,
        occupancy: UiIntentOccupancyPosture::Idle,
        policy: UiIntentPolicyPosture::Admitted,
        affinity: UiIntentAffinityPosture::Current,
        confirmation: UiIntentConfirmationPosture::NotRequired,
        selected_dependencies_visited: 0,
    })
}

#[test]
fn condition_reobservation_refreshes_only_existing_facts_of_the_named_routes() {
    let mut owner = UiIntentOperabilityStandingOwner::default();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let instances: Vec<_> = (0..3)
        .map(|_| UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect();
    for (index, instance) in instances.iter().enumerate() {
        owner.replace_instance(*instance, row(*instance, binding, 1 + index % 2, 1));
    }
    owner.revision = 1;
    assert_eq!(owner.route_members("route.0"), 3);
    assert_eq!(owner.route_members("route.1"), 1);
    let ready = UiIntentOperabilityDecision::ready_for_test();
    let before = owner.snapshot();

    assert_eq!(
        owner.reobserve(["route.1", "missing"], |_| Some(ready.clone())),
        1,
        "each existing fact of a named route is one attempt; a missing route is none"
    );
    assert_eq!(owner.reobserve(["route.0"], |_| None), 3);
    assert_eq!(
        owner.snapshot(),
        before,
        "an equal or unavailable decision leaves the owner and its revision alone"
    );

    let readonly = readonly();
    assert_eq!(owner.reobserve(["route.1"], |_| Some(readonly.clone())), 1);
    let after = owner.snapshot();
    assert_eq!(after.owner_revision(), 2);
    assert_eq!(after.changed_instances(&before).as_ref(), &[instances[1]]);
    assert_eq!(after.facts().len(), before.facts().len());
    let graph = crate::graph::UiGraphNodeIdentity::new(71);
    let refreshed = after.fact_for(graph, instances[1], "route.1").unwrap();
    assert_eq!(refreshed.decision(), &readonly);
    assert_eq!(refreshed.owner_revision(), 2);
    assert_eq!(
        after
            .fact_for(graph, instances[1], "route.0")
            .unwrap()
            .decision(),
        &ready
    );

    owner.retire_instance(instances[1]);
    assert_eq!(owner.route_members("route.1"), 0);
    assert_eq!(owner.route_members("route.0"), 2);
    owner.retire_binding(binding);
    assert_eq!(owner.route_members("route.0"), 0);
    assert_eq!(
        owner.reobserve(["route.0", "route.1"], |_| Some(ready.clone())),
        0
    );
}

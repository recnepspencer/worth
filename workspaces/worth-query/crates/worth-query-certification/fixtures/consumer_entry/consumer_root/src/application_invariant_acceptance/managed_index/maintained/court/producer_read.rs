//! The generated producer's own coupled decision reads the managed Parent:
//! at its own scope on the applied candidate's aligned basis, and at a peer
//! scope sharing its watch grant. An unrelated scope, an unpublished Source
//! change and a cancelled request each refuse before or inside the read,
//! typed, and nothing publishes.
use super::super::generated::{program, source};
use super::super::peer_scope::{
    ProbeRefusal, ProducerProbe, OBSERVED, PRODUCER_PROBE, SEED_PEERS, SOURCE_CREATES,
};
use super::super::{authoring::Edit, host as world_host};
use super::*;

struct Run {
    performed: bool,
    observed: Vec<super::super::peer_scope::Observed>,
    probes_during_run: usize,
}

fn fresh_world() -> (world_host::Host, Arc<prefix_observation::Calls>) {
    let provider = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    SEED_PEERS.set(true);
    let world = world_host::install(Arc::clone(&provider), rules);
    SEED_PEERS.set(false);
    (world, provider)
}

/// Authors one Item into `directory` through the ordinary program.
fn author(world: &world_host::Host, directory: &str, identity: &str, key: u64) {
    let scope = authentication::request_scope();
    let principal = principal(world, &scope);
    let outcome = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(Edit::Item {
            directory: directory.into(),
            identity: identity.into(),
            name: identity.into(),
        })
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

fn principal(
    world: &world_host::Host,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<super::super::schema::Catalog> {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}

/// One genuine source action in `catalog`; its producer decision probes.
fn run(
    world: &world_host::Host,
    provider: &prefix_observation::Calls,
    probe: ProducerProbe,
    creates: Option<&'static str>,
    scope: &WorthQueryRequestScope,
    key: u64,
) -> Run {
    let opened = authentication::request_scope();
    let principal = principal(world, &opened);
    let reader = world
        .runtime()
        .request(&principal, &opened)
        .on_branch(world.current_world());
    let handles: Vec<_> = ["catalog", "peer", "A"]
        .into_iter()
        .map(|directory| {
            reader
                .query(find(directory, ""))
                .open_index::<prefix::PrefixIndex>()
                .unwrap()
        })
        .collect();
    let probes = provider.probe_validations.load(Ordering::SeqCst);
    OBSERVED.with(|all| all.borrow_mut().clear());
    PRODUCER_PROBE.set(Some(probe));
    SOURCE_CREATES.set(creates);
    let performed = world
        .runtime()
        .request(&principal, scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(source::operation::RequestItem {
            directory: "catalog".into(),
            name: format!("probe-{key}"),
        })
        .without_source()
        .idempotency(&key)
        .execute_performed::<world_host::Program, program::Root>(
            world,
            ExecutionAllocationPolicy::SystemAllocation,
        );
    PRODUCER_PROBE.set(None);
    SOURCE_CREATES.set(None);
    let probes_during_run = provider.probe_validations.load(Ordering::SeqCst) - probes;
    for handle in handles {
        handle.close();
    }
    Run {
        performed: matches!(
            performed,
            Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_))
        ),
        observed: OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
        probes_during_run,
    }
}

#[test]
fn producer_decision_reads_its_own_and_a_shared_owner_peer_parent_on_the_aligned_basis() {
    lifecycle::run(|| {
        // The producer runs on its source's initial lifecycle: one world each.
        let scope = authentication::request_scope();
        let (world, provider) = fresh_world();
        author(&world, "catalog", "c", 0x2820_0001);
        let own = run(&world, &provider, ProducerProbe::Own, None, &scope, 0x2820_0010);
        assert_eq!(own.observed, vec![Ok(vec!["c".to_owned()])]);
        assert!(own.performed, "the aligned producer decision publishes");
        let (world, provider) = fresh_world();
        author(&world, "peer", "p", 0x2820_0002);
        let peer = run(&world, &provider, ProducerProbe::Peer("peer"), None, &scope, 0x2820_0011);
        assert_eq!(peer.observed, vec![Ok(vec!["p".to_owned()])]);
        assert!(peer.performed, "the declared shared-owner peer read publishes");
    });
}

#[test]
fn producer_decision_refuses_an_unrelated_scope_and_an_unpublished_source_change_before_the_provider() {
    lifecycle::run(|| {
        let scope = authentication::request_scope();
        // `A` shares no watch grant with `catalog`: not reachable as a peer.
        let (world, provider) = fresh_world();
        let unrelated = run(&world, &provider, ProducerProbe::Peer("A"), None, &scope, 0x2820_0020);
        assert_eq!(
            unrelated.observed,
            vec![Err(ProbeRefusal::Refused(
                WorthQueryManagedIndexRefusal::ForeignAssociation
            ))]
        );
        // The source action's own new Item is an unpublished change that the
        // Parent's Source sees: the admitted before basis is not its image.
        let (world, provider) = fresh_world();
        let unaligned = run(&world, &provider, ProducerProbe::Own, Some("fresh"), &scope, 0x2820_0021);
        assert_eq!(
            unaligned.observed,
            vec![Err(ProbeRefusal::Refused(
                WorthQueryManagedIndexRefusal::UnpublishedSourceChange
            ))]
        );
        for refused in [unrelated, unaligned] {
            assert!(!refused.performed, "a refused producer read publishes nothing");
            assert_eq!(refused.probes_during_run, 0, "refusal precedes every provider probe");
        }
    });
}

#[test]
fn producer_decision_peer_read_carries_original_request_cancellation() {
    lifecycle::run(|| {
        let (world, provider) = fresh_world();
        author(&world, "peer", "p", 0x2820_0030);
        let cancellation = WorthQueryCancellationSource::new();
        let stopped = WorthQueryRequestScope::new(
            None,
            cancellation.token(),
            WorthQueryAdmissionClock::system(),
        );
        // The provider's genuine probe of the peer cancels the original request.
        *provider.at_probe.lock().unwrap() =
            Some((String::new(), Box::new(move || cancellation.cancel())));
        let run = run(&world, &provider, ProducerProbe::Peer("peer"), None, &stopped, 0x2820_0031);
        assert!(
            provider.at_probe.lock().unwrap().is_none(),
            "the producer's own peer read reached the provider"
        );
        assert_eq!(run.observed, vec![Err(ProbeRefusal::Cancelled)]);
        assert!(!run.performed, "the cancelled request publishes nothing");
    });
}

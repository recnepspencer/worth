//! The generated producer probes its own or a declared peer's managed Parent
//! as part of a real coupled decision or partitioned computation.
use super::super::generated::{program, source};
use super::super::peer_scope::{
    ProbeRefusal, ProducerProbe, OBSERVED, ORDERED_SUCCESSOR, PRODUCER_PROBE, SEED_PEERS,
    SOURCE_CREATES, SOURCE_SIBLING, THROUGH_COMPUTATION,
};
use super::super::{authoring::Edit, host as world_host};
use super::*;

mod overlay;

struct Run {
    performed: bool,
    refusal: Option<String>,
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
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "seed {directory}/{identity}: {outcome:?}"
    );
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
    SOURCE_SIBLING.set(None);
    ORDERED_SUCCESSOR.set(None);
    THROUGH_COMPUTATION.set(false);
    let probes_during_run = provider.probe_validations.load(Ordering::SeqCst) - probes;
    for handle in handles {
        handle.close();
    }
    let refusal = match &performed {
        Err(stop) => Some(format!("{stop:?}")),
        Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(stop)) => {
            Some(format!("{stop:?}"))
        }
        Ok(_) => None,
    };
    Run {
        refusal,
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
        let own = run(
            &world,
            &provider,
            ProducerProbe::Own,
            None,
            &scope,
            0x2820_0010,
        );
        assert_eq!(own.observed, vec![Ok(vec!["c".to_owned()])]);
        assert!(own.performed, "the aligned producer decision publishes");
        let (world, provider) = fresh_world();
        author(&world, "peer", "p", 0x2820_0002);
        let peer = run(
            &world,
            &provider,
            ProducerProbe::Peer("peer"),
            None,
            &scope,
            0x2820_0011,
        );
        assert_eq!(peer.observed, vec![Ok(vec!["p".to_owned()])]);
        assert!(
            peer.performed,
            "the declared shared-owner peer read publishes"
        );
    });
}

#[test]
fn producer_decision_reads_an_unpublished_own_row_and_refuses_an_unrelated_scope() {
    lifecycle::run(|| {
        let scope = authentication::request_scope();
        let (world, provider) = fresh_world();
        let unrelated = run(
            &world,
            &provider,
            ProducerProbe::Peer("A"),
            None,
            &scope,
            0x2820_0020,
        );
        assert_eq!(
            unrelated.observed,
            vec![Err(ProbeRefusal::Refused(
                WorthQueryManagedIndexRefusal::ForeignAssociation
            ))]
        );
        assert!(!unrelated.performed);
        assert_eq!(unrelated.probes_during_run, 0);

        let (world, provider) = fresh_world();
        let changed = run(
            &world,
            &provider,
            ProducerProbe::Own,
            Some("fresh"),
            &scope,
            0x2820_0021,
        );
        assert_eq!(changed.observed, vec![Ok(vec!["fresh".to_owned()])]);
        assert!(changed.performed);
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
        let run = run(
            &world,
            &provider,
            ProducerProbe::Peer("peer"),
            None,
            &stopped,
            0x2820_0031,
        );
        assert!(
            provider.at_probe.lock().unwrap().is_none(),
            "the producer's own peer read reached the provider"
        );
        assert_eq!(run.observed, vec![Err(ProbeRefusal::Cancelled)]);
        assert!(!run.performed, "the cancelled request publishes nothing");
    });
}

#[test]
fn producer_peer_read_keeps_the_peer_source_when_the_own_source_changes() {
    lifecycle::run(|| {
        let scope = authentication::request_scope();
        let (world, provider) = fresh_world();
        author(&world, "peer", "p", 0x2820_0040);
        SOURCE_SIBLING.set(Some("sibling"));
        let sibling = run(
            &world,
            &provider,
            ProducerProbe::Peer("peer"),
            None,
            &scope,
            0x2820_0041,
        );
        assert_eq!(sibling.observed, vec![Ok(vec!["p".to_owned()])]);
        assert!(sibling.performed);

        let (world, provider) = fresh_world();
        author(&world, "peer", "p", 0x2820_0042);
        SOURCE_SIBLING.set(Some("sibling"));
        let row = run(
            &world,
            &provider,
            ProducerProbe::Peer("peer"),
            Some("fresh"),
            &scope,
            0x2820_0043,
        );
        assert_eq!(row.observed, vec![Ok(vec!["p".to_owned()])]);
        assert!(row.performed);
    });
}

#[test]
fn producer_computation_reads_own_and_peer_parents_and_refuses_before_the_provider() {
    lifecycle::run(|| {
        let scope = authentication::request_scope();
        let computed = |probe, creates, own: Option<(&str, &str)>, key| {
            let (world, provider) = fresh_world();
            if let Some((directory, identity)) = own {
                author(&world, directory, identity, key);
            }
            THROUGH_COMPUTATION.set(true);
            run(&world, &provider, probe, creates, &scope, key + 1)
        };
        let own = computed(
            ProducerProbe::Own,
            None,
            Some(("catalog", "c")),
            0x2820_0050,
        );
        assert_eq!(own.observed, vec![Ok(vec!["c".to_owned()])]);
        assert!(
            own.performed,
            "the computation's aligned own read publishes"
        );
        let peer = computed(
            ProducerProbe::Peer("peer"),
            None,
            Some(("peer", "p")),
            0x2820_0052,
        );
        assert_eq!(peer.observed, vec![Ok(vec!["p".to_owned()])]);
        assert!(
            peer.performed,
            "the computation's declared peer read publishes"
        );
        let changed = computed(ProducerProbe::Own, Some("fresh"), None, 0x2820_0054);
        assert_eq!(changed.observed, vec![Ok(vec!["fresh".to_owned()])]);
        assert!(changed.performed);
        let refused = computed(ProducerProbe::Peer("A"), None, None, 0x2820_0060);
        assert_eq!(
            refused.observed,
            vec![Err(ProbeRefusal::Refused(
                WorthQueryManagedIndexRefusal::ForeignAssociation
            ))]
        );
        assert!(!refused.performed);
        assert_eq!(refused.probes_during_run, 0);
        let undeclared = computed(ProducerProbe::Undeclared, Some("fresh"), None, 0x2820_0070);
        assert_eq!(undeclared.observed, vec![Err(ProbeRefusal::Undeclared)]);
        assert!(!undeclared.performed);
        assert_eq!(undeclared.probes_during_run, 0);
    });
}

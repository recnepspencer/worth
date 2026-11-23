//! An ordinary operation's decision reads the managed Parent at a declared
//! shared-owner peer scope on the committed basis. An unrelated scope and a
//! Parent declared only for the operation's own scope refuse before any
//! provider probe, and a committed edit to the peer's Source after the read
//! turns the commit Stale instead of publishing.
use super::super::peer_scope::{ProbeRefusal, AFTER_READ, OBSERVED, SEED_PEERS};
use super::super::{authoring::Edit, handler::DecisionRead};
use super::*;
use std::{sync::mpsc, time::Duration};

fn principal(
    world: &host::Host,
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

fn author(world: &host::Host, directory: &str, identity: &str, key: u64) {
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
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation);
    assert!(matches!(
        outcome,
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ));
}

/// A peer world: `catalog` holds `c`, the grant-sharing `peer` holds `p`.
fn peer_world(read: DecisionRead) -> (host::Host, Arc<prefix_observation::Calls>) {
    let provider = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    SEED_PEERS.set(true);
    let world = host::install_decision_read(Arc::clone(&provider), rules, read);
    SEED_PEERS.set(false);
    author(&world, "catalog", "c", 0x2830_0001);
    author(&world, "peer", "p", 0x2830_0002);
    (world, provider)
}

/// Renames `c` in `catalog`; the decision reads as `world` was configured.
/// `probes` counts the provider probes the mutation itself made.
fn rename_in_catalog(
    world: &host::Host,
    key: u64,
    probes: Option<(&prefix_observation::Calls, &mut usize)>,
) -> Result<
    WorthQueryApplicationMutationOutcome<mutation::Denial, mutation::Renamed>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let scope = authentication::request_scope();
    let principal = principal(world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = request.query(find("catalog", "c")).execute().unwrap();
    let before = probes
        .as_ref()
        .map(|(provider, _)| provider.probe_validations.load(Ordering::SeqCst));
    let result = request
        .advanced()
        .mutate(mutation::Rename {
            directory: "catalog".into(),
            item: "c".into(),
            prefix: "c".into(),
            replacement: "cz".into(),
        })
        .expect_result_set(source.result_set_observation().unwrap().clone())
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation);
    if let (Some((provider, during)), Some(before)) = (probes, before) {
        *during = provider.probe_validations.load(Ordering::SeqCst) - before;
    }
    result
}

fn observed() -> Vec<super::super::peer_scope::Observed> {
    OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut()))
}

#[test]
fn operation_decision_reads_a_declared_shared_owner_peer_and_refuses_unrelated_or_undeclared_scopes() {
    lifecycle::run(|| {
        let (world, _) = peer_world(DecisionRead::Peer("peer"));
        observed();
        let outcome = rename_in_catalog(&world, 0x2830_0010, None).unwrap();
        assert_eq!(observed(), vec![Ok(vec!["p".to_owned()])]);
        assert!(
            matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
            "{outcome:?}"
        );
        // `A` shares no grant with `catalog`; the Exact Parent is declared for
        // the operation's own scope only.
        for read in [DecisionRead::Peer("A"), DecisionRead::UndeclaredPeer("peer")] {
            let (world, provider) = peer_world(read);
            let mut probes = usize::MAX;
            observed();
            let result = rename_in_catalog(&world, 0x2830_0020, Some((&provider, &mut probes)));
            assert_eq!(
                observed(),
                vec![Err(ProbeRefusal::Refused(
                    WorthQueryManagedIndexRefusal::ForeignAssociation
                ))]
            );
            assert!(
                matches!(
                    result,
                    Err(WorthQueryApplicationRequestMutationDenial::Handler(
                        MutationHandlerExecutionDenial::Handler(_)
                    ))
                ),
                "{result:?}"
            );
            assert_eq!(probes, 0, "scope authority precedes every provider probe");
        }
    });
}

#[test]
fn operation_peer_read_then_committed_peer_edit_cannot_publish_the_stale_decision() {
    lifecycle::run(|| {
        let (world, _) = peer_world(DecisionRead::Peer("peer"));
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (released_tx, released_rx) = mpsc::sync_channel(1);
        // The peer read completed; the decision is held before its commit.
        AFTER_READ.with(|hold| {
            *hold.borrow_mut() = Some(Box::new(move || {
                entered_tx.send(()).unwrap();
                released_rx
                    .recv_timeout(Duration::from_secs(10))
                    .expect("the independent writer commits while the decision is held");
            }))
        });
        observed();
        let world = &world;
        let result = std::thread::scope(|threads| {
            let writer = threads.spawn(move || {
                entered_rx
                    .recv_timeout(Duration::from_secs(10))
                    .expect("the decision read the peer");
                // An independent entrant in the peer's Source commits.
                author(world, "peer", "q", 0x2830_0030);
                released_tx.send(()).unwrap();
            });
            let result = rename_in_catalog(world, 0x2830_0031, None);
            writer.join().unwrap();
            result
        });
        assert_eq!(observed(), vec![Ok(vec!["p".to_owned()])]);
        match result {
            Ok(WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::Stale(stale),
            )) => assert!(stale.stale_fact_count() >= 1, "the peer's Source fact moved"),
            other => panic!("the stale peer read must not publish: {other:?}"),
        }
    });
}

#[test]
fn operation_peer_read_admits_a_peer_through_any_declared_governing_relation() {
    lifecycle::run(|| {
        // `shelved` shares no watch grant with `catalog`, only the shelf: the
        // second declared relation for the same Parent reaches it.
        let (world, _) = peer_world(DecisionRead::Peer("shelved"));
        author(&world, "shelved", "s", 0x2830_0040);
        observed();
        let outcome = rename_in_catalog(&world, 0x2830_0041, None).unwrap();
        assert_eq!(observed(), vec![Ok(vec!["s".to_owned()])]);
        assert!(
            matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
            "{outcome:?}"
        );
    });
}

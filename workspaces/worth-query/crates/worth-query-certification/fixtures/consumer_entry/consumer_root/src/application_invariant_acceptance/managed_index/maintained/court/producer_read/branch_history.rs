//! A closed fork releases its World history whichever basis the producer's
//! managed read used: the committed index for a scope the envelope left
//! alone, or the applied candidate for a scope it created or changed. The
//! facts a settled output retains name the occurrence they read; none holds it.
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;

/// What one fork observed and how its retirement ended.
struct Closed {
    observed: Vec<super::super::super::peer_scope::Observed>,
    committed: Result<(), String>,
    /// The fork's close, with its pending owner cleanup retried once.
    released: Result<(), String>,
    pending_cleanups: usize,
}

/// What the source action's envelope does before its producer probes.
struct Envelope {
    stages: Vec<Edit>,
    probe: ProducerProbe,
    /// The source action itself also creates this Item in `catalog`.
    creates: Option<&'static str>,
}

/// On a fork: `late`, a peer of `catalog`, is created by one commit and
/// filled by the next beside `peer`. A third commit runs `envelope` and the
/// source action in `catalog` whose producer probes; then the fork closes.
fn close_after(envelope: Envelope, computation: bool, key: u64) -> Closed {
    let (world, _) = fresh_world();
    let branch = fork(&world);
    let late = Edit::GrantedDirectory {
        identity: "late".into(),
    };
    edit_on(&world, branch, late, key);
    for (directory, edit_key) in [("late", key + 1), ("peer", key + 2)] {
        edit_on(&world, branch, item(directory, "p"), edit_key);
    }
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let mut edit = request
        .edit_envelope()
        .idempotency(key + 3)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    for stage in envelope.stages {
        edit.push_stage::<ApplicationNoOutputGraph, _>(stage)
            .unwrap();
    }
    edit.push_stage::<program::Root, _>(source::operation::RequestItem {
        directory: "catalog".into(),
        name: "producer-output".into(),
    })
    .unwrap();
    OBSERVED.with(|all| all.borrow_mut().clear());
    PRODUCER_PROBE.set(Some(envelope.probe));
    SOURCE_CREATES.set(envelope.creates);
    THROUGH_COMPUTATION.set(computation);
    let outcome = edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation);
    PRODUCER_PROBE.set(None);
    SOURCE_CREATES.set(None);
    THROUGH_COMPUTATION.set(false);
    let committed = match outcome {
        // The receipt holds its commit; it is released before the close.
        Ok(WorthQueryEditEnvelopeOutcome::Committed(receipt)) => {
            drop(receipt);
            Ok(())
        }
        Ok(WorthQueryEditEnvelopeOutcome::Uncommitted(cause)) => Err(format!("{cause:?}")),
        Ok(WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause)) => {
            Err(format!("stage {}: {:?}", cause.binding(), cause.cause()))
        }
        Ok(other) => Err(format!("{:?}", std::mem::discriminant(&other))),
        Err(cause) => Err(format!("{cause:?}")),
    };
    let (released, pending_cleanups) = close(&world, branch);
    Closed {
        observed: OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
        committed,
        released,
        pending_cleanups,
    }
}

type Branch = worth_query_host::facade::product::WorthQueryProductBranch;

fn fork(world: &world_host::Host) -> Branch {
    world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap()
}

/// Closes `branch`, retrying its pending owner cleanup once, and counts the
/// cleanups the runtime still holds afterwards.
fn close(world: &world_host::Host, branch: Branch) -> (Result<(), String>, usize) {
    let closed = world
        .runtime()
        .with_host_advancement(|phase| world.runtime().on_branch(branch).close(&phase))
        .unwrap();
    let released = match closed {
        Ok(_) => Ok(()),
        Err(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(pending)) => pending
            .into_cleanup()
            .retry()
            .map(drop)
            .map_err(|failure| format!("{:?}", failure.denial())),
        Err(denial) => Err(format!("{denial:?}")),
    };
    (released, world.runtime().branches().pending_cleanup().len())
}

fn item(directory: &str, identity: &str) -> Edit {
    Edit::Item {
        directory: directory.into(),
        identity: identity.into(),
        name: identity.into(),
    }
}

/// Both readers of the producer commit, observe `rows` and leave a fork that
/// releases all of its history at its close.
fn releases(envelope: fn() -> Envelope, rows: &[&str], key: u64) {
    for (computation, reader) in [(false, "decision"), (true, "computation")] {
        let closed = close_after(envelope(), computation, key + u64::from(computation) * 0x10);
        assert_eq!(closed.committed, Ok(()), "{reader}");
        assert_eq!(
            closed.observed,
            vec![Ok(rows.iter().map(|row| (*row).to_owned()).collect())],
            "{reader}"
        );
        assert_eq!(closed.released, Ok(()), "{reader}");
        assert_eq!(closed.pending_cleanups, 0, "{reader}");
    }
}

#[test]
fn a_closed_fork_releases_its_history_after_a_committed_index_read_of_an_untouched_peer() {
    lifecycle::run(|| {
        let untouched = || Envelope {
            stages: vec![],
            probe: ProducerProbe::Peer("peer"),
            creates: None,
        };
        releases(untouched, &["p"], 0x3105_0010);
    });
}

#[test]
fn a_closed_fork_releases_its_history_after_an_applied_read_of_a_scope_created_in_the_envelope() {
    lifecycle::run(|| {
        let created = || Envelope {
            stages: vec![
                Edit::GrantedDirectory {
                    identity: "novel".into(),
                },
                item("novel", "fresh"),
            ],
            probe: ProducerProbe::Peer("novel"),
            creates: None,
        };
        releases(created, &["fresh"], 0x3105_0040);
    });
}

#[test]
fn a_closed_fork_releases_its_history_after_an_applied_read_of_a_scope_changed_in_the_envelope() {
    lifecycle::run(|| {
        // `late` was committed on the fork before the envelope changes it.
        let changed = || Envelope {
            stages: vec![item("late", "fresh")],
            probe: ProducerProbe::Peer("late"),
            creates: None,
        };
        releases(changed, &["fresh", "p"], 0x3105_0080);
        // The producer's own committed scope, changed by its source action.
        let own = || Envelope {
            stages: vec![],
            probe: ProducerProbe::Own,
            creates: Some("fresh"),
        };
        releases(own, &["fresh"], 0x3105_00c0);
    });
}

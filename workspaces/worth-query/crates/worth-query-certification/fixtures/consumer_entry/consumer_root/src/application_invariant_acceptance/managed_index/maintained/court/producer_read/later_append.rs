//! A later append in the same envelope changes what a producer probed. An
//! aligned read from the committed basis keeps candidate support: a later
//! member that changes its Source is found at that append, so the stale
//! decision never publishes beside the edit that contradicts it.
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;

struct Later {
    observed: Vec<super::super::super::peer_scope::Observed>,
    committed: bool,
    detail: String,
    rows: Vec<String>,
}

/// The source action in `catalog` (its producer probes), then one `later`
/// authored stage; `rows` reads `directory` back.
fn later_stage(
    probe: ProducerProbe,
    computation: bool,
    later: Edit,
    directory: &'static str,
    key: u64,
) -> Later {
    let (world, _) = fresh_world();
    author(&world, "catalog", "c", key);
    author(&world, "peer", "p", key + 1);
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let mut envelope = request
        .edit_envelope()
        .idempotency(key + 2)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    envelope
        .push_stage::<program::Root, _>(source::operation::RequestItem {
            directory: "catalog".into(),
            name: "producer-output".into(),
        })
        .unwrap();
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(later)
        .unwrap();
    OBSERVED.with(|all| all.borrow_mut().clear());
    PRODUCER_PROBE.set(Some(probe));
    THROUGH_COMPUTATION.set(computation);
    let outcome = envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation);
    PRODUCER_PROBE.set(None);
    THROUGH_COMPUTATION.set(false);
    let (committed, detail) = match outcome {
        Ok(WorthQueryEditEnvelopeOutcome::Committed(_)) => (true, String::new()),
        Ok(WorthQueryEditEnvelopeOutcome::Uncommitted(cause)) => (false, format!("{cause:?}")),
        Ok(WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause)) => (
            false,
            format!("stage {}: {:?}", cause.binding(), cause.cause()),
        ),
        Ok(other) => (false, format!("{:?}", std::mem::discriminant(&other))),
        Err(cause) => (false, format!("{cause:?}")),
    };
    let rows = request
        .query(find(directory, ""))
        .execute()
        .unwrap()
        .rows()
        .iter()
        .map(|row| row.identity.clone())
        .collect();
    Later {
        observed: OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
        committed,
        detail,
        rows,
    }
}

fn item(directory: &str) -> Edit {
    Edit::Item {
        directory: directory.into(),
        identity: "late".into(),
        name: "late".into(),
    }
}

/// The typed final-support refusal naming the managed Source read.
fn refused_support(run: &Later) {
    assert!(!run.committed, "the stale decision published: {:?}", run.observed);
    assert!(
        run.detail.contains("CoupledReadSupportChanged")
            && run.detail.contains(
                "a later coupled contribution changed decision support: application-managed-source"
            ),
        "{}",
        run.detail
    );
}

#[test]
fn an_aligned_read_refuses_when_a_later_stage_edits_the_probed_source() {
    lifecycle::run(|| {
        for computation in [false, true] {
            let key = 0x3101_0010 + u64::from(computation) * 0x20;
            // The producer read the unchanged peer from the committed index
            // at its own position; the later stage then gave the peer an Item.
            let peer = later_stage(
                ProducerProbe::Peer("peer"),
                computation,
                item("peer"),
                "peer",
                key,
            );
            assert_eq!(peer.observed, vec![Ok(vec!["p".to_owned()])]);
            refused_support(&peer);
            assert_eq!(peer.rows, ["p"], "nothing published");
            // The same for its own Directory, after its own output.
            let own = later_stage(
                ProducerProbe::Own,
                computation,
                item("catalog"),
                "catalog",
                key + 8,
            );
            assert_eq!(own.observed, vec![Ok(vec!["c".to_owned()])]);
            refused_support(&own);
            assert_eq!(own.rows, ["c"], "nothing published");
        }
    });
}

#[test]
fn an_aligned_read_commits_when_a_later_stage_writes_the_same_kinds_under_another_scope() {
    lifecycle::run(|| {
        for computation in [false, true] {
            let key = 0x3101_0060 + u64::from(computation) * 0x20;
            // The later stage gives the unrelated Directory `A` an Item: the
            // Source's own kinds and relation, but nothing the probed scope's
            // Source read. Neither aligned decision is disturbed.
            let peer = later_stage(ProducerProbe::Peer("peer"), computation, item("A"), "peer", key);
            assert_eq!(peer.observed, vec![Ok(vec!["p".to_owned()])]);
            assert!(peer.committed, "{}", peer.detail);
            assert_eq!(peer.rows, ["p"]);
            let own = later_stage(ProducerProbe::Own, computation, item("A"), "A", key + 8);
            assert_eq!(own.observed, vec![Ok(vec!["c".to_owned()])]);
            assert!(own.committed, "{}", own.detail);
            assert_eq!(own.rows, ["late", "x"]);
            // A later stage that only creates another Directory, likewise.
            let other = later_stage(
                ProducerProbe::Peer("peer"),
                computation,
                Edit::Directory {
                    identity: "elsewhere".into(),
                },
                "peer",
                key + 16,
            );
            assert_eq!(other.observed, vec![Ok(vec!["p".to_owned()])]);
            assert!(other.committed, "{}", other.detail);
        }
    });
}

//! The probed scope key resolves on the applied candidate: a scope an
//! earlier stage retired is unknown, and a key an earlier stage gave to a
//! new Directory names that Directory, never the record it named at C.
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;

/// Earlier authored `stages`, then the source action in `catalog` whose
/// producer probes the peer named `probed`.
fn probe_after(
    stages: Vec<Edit>,
    probed: &'static str,
    computation: bool,
    key: u64,
) -> (Vec<super::super::super::peer_scope::Observed>, bool, String) {
    let (world, _) = fresh_world();
    author(&world, "peer", "p", key);
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let mut envelope = request
        .edit_envelope()
        .idempotency(key + 1)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    for stage in stages {
        envelope
            .push_stage::<ApplicationNoOutputGraph, _>(stage)
            .unwrap();
    }
    envelope
        .push_stage::<program::Root, _>(source::operation::RequestItem {
            directory: "catalog".into(),
            name: "producer-output".into(),
        })
        .unwrap();
    OBSERVED.with(|all| all.borrow_mut().clear());
    PRODUCER_PROBE.set(Some(ProducerProbe::Peer(probed)));
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
    (
        OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
        committed,
        detail,
    )
}

#[test]
fn a_scope_retired_earlier_in_the_envelope_is_unknown_to_the_probe() {
    lifecycle::run(|| {
        for computation in [false, true] {
            let key = 0x3102_0010 + u64::from(computation) * 0x10;
            // Live at C, retired by the earlier stage.
            let (observed, committed, detail) = probe_after(
                vec![Edit::RetireDirectory {
                    directory: "peer".into(),
                }],
                "peer",
                computation,
                key,
            );
            assert_eq!(observed, vec![Err(ProbeRefusal::UnknownScope)], "{detail}");
            assert!(!committed);
        }
    });
}

#[test]
fn a_key_given_to_a_new_scope_earlier_in_the_envelope_reads_the_new_scope() {
    lifecycle::run(|| {
        for computation in [false, true] {
            // At C `peer` names a Directory holding `p`. The envelope retires
            // it, creates a new `peer` under the watch grant (a governing
            // relation linked in this envelope) and gives it `fresh`.
            let (observed, committed, detail) = probe_after(
                vec![
                    Edit::RetireDirectory {
                        directory: "peer".into(),
                    },
                    Edit::GrantedDirectory {
                        identity: "peer".into(),
                    },
                    Edit::Item {
                        directory: "peer".into(),
                        identity: "fresh".into(),
                        name: "fresh".into(),
                    },
                ],
                "peer",
                computation,
                0x3102_0040 + u64::from(computation) * 0x10,
            );
            assert_eq!(observed, vec![Ok(vec!["fresh".to_owned()])], "{detail}");
            assert!(committed, "{detail}");
        }
    });
}

#[test]
fn a_peer_whose_governing_relation_was_unlinked_earlier_in_the_envelope_is_refused() {
    lifecycle::run(|| {
        for computation in [false, true] {
            // At C the watch grant makes `peer` a declared peer of `catalog`.
            // The earlier stage unlinks it; the applied candidate has no
            // shared owner, so the probe refuses before any provider work.
            let (observed, committed, detail) = probe_after(
                vec![Edit::UngrantDirectory {
                    directory: "peer".into(),
                }],
                "peer",
                computation,
                0x3102_0080 + u64::from(computation) * 0x10,
            );
            assert_eq!(
                observed,
                vec![Err(ProbeRefusal::Refused(
                    WorthQueryManagedIndexRefusal::ForeignAssociation
                ))],
                "{detail}"
            );
            assert!(!committed);
        }
    });
}

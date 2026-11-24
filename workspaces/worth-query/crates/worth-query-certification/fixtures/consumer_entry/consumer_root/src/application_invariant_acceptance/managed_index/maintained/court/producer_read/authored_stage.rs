//! A later authored stage's own decision probes the applied candidate: it
//! selects an Item an earlier stage of the same envelope created.
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;

#[test]
fn a_later_authored_stage_probes_an_item_an_earlier_stage_created() {
    lifecycle::run(|| {
        let (world, _) = fresh_world();
        let scope = authentication::request_scope();
        let principal = principal(&world, &scope);
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let mut envelope = request
            .edit_envelope()
            .idempotency(0x3103_0010u64)
            .runtime_stages(&world)
            .ordered_authored()
            .unwrap();
        for identity in ["newcomer", "second"] {
            envelope
                .push_stage::<ApplicationNoOutputGraph, _>(Edit::Item {
                    directory: "catalog".into(),
                    identity: identity.into(),
                    name: identity.into(),
                })
                .unwrap();
        }
        OBSERVED.with(|all| all.borrow_mut().clear());
        super::super::super::peer_scope::STAGE_PROBE.set(Some("catalog"));
        let outcome = envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation);
        super::super::super::peer_scope::STAGE_PROBE.set(None);
        match outcome.unwrap() {
            WorthQueryEditEnvelopeOutcome::Committed(_) => {}
            WorthQueryEditEnvelopeOutcome::Uncommitted(cause) => panic!("uncommitted: {cause:?}"),
            WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause) => {
                panic!("stage {} refused: {:?}", cause.binding(), cause.cause())
            }
            other => panic!("unexpected outcome: {:?}", std::mem::discriminant(&other)),
        }
        // The first stage decides on the committed basis; the second stage's
        // own decision probes the applied candidate and selects the Item the
        // first stage created, before anything is published.
        assert_eq!(
            OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
            vec![Ok(vec![]), Ok(vec!["newcomer".to_owned()])]
        );
        let rows = request.query(find("catalog", "")).execute().unwrap();
        assert_eq!(
            rows.rows()
                .iter()
                .map(|row| row.identity.as_str())
                .collect::<Vec<_>>(),
            ["newcomer", "second"]
        );
    });
}

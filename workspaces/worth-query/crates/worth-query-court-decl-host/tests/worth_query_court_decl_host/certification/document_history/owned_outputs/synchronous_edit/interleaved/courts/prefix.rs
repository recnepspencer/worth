//! An ordered prefix reads actual C, never a future required output.
use super::*;

#[test]
fn positioned_prefix_keeps_current_reads_future_absence_and_final_support() {
    let _execution = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for (seed, change) in [(true, false), (false, true), (true, true)] {
        let host = host(None, None);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        if seed {
            let outcome = request
                .edit_envelope()
                .stage::<ModelTree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: 1,
                    key_derived_target: false,
                })
                .idempotency("published-first".to_owned())
                .execute_performed::<Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
            assert_eq!(
                request
                    .query(query::first_output::Read("first:a".into()))
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                10
            );
        }
        let before = super::super::super::courts::native_head(&host);
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("prefix-current:{seed}:{change}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        let prefix = edit
            .push_stage::<ProducedTree, _>(dependent::Intent { add: 7 })
            .unwrap();
        if change {
            edit.push_stage::<ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 3,
                key_derived_target: false,
            })
            .unwrap();
        }
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (result, writes) = observe_maintenance_work(24, || {
            edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        });
        let publications = writes.published_rows.len() + writes.omitted_publications;
        if !seed {
            let failure = result.expect_err("a future root cannot supply the prefix read");
            use worth_query_host::facade::application_entry::{
                WorthQueryApplicationRequestMutationDenial as Mutation,
                WorthQueryPerformedMutationExecutionDenial as Performed,
            };
            let Performed::Mutation(Mutation::Handler(MutationHandlerExecutionDenial::Handler(
                cause,
            ))) = failure.cause()
            else {
                panic!("{failure:?}")
            };
            let refusal = std::error::Error::source(cause)
                .unwrap()
                .downcast_ref::<WorthQueryEntityResolutionDenial>()
                .unwrap();
            assert_eq!(
                refusal.kind(),
                WorthQueryEntityResolutionDenialKind::Refused(
                    WorthQueryEntityResolutionRefusalKind::UnknownEntity
                )
            );
            assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
            assert_eq!(publications, 0);
        } else if change {
            let outcome = result.unwrap();
            let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome
            else {
                panic!("{outcome:?}")
            };
            assert_eq!(
                cause.invariant_execution_failure().unwrap().kind(),
                WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
            );
            assert_eq!(publications, 0);
        } else {
            let outcome = result.unwrap();
            let Outcome::Committed(receipt) = outcome else {
                panic!("{outcome:?}")
            };
            let value = receipt.result(&prefix).unwrap();
            assert_eq!((value.value, value.ordinal), (17, 0));
            assert_eq!(publications, 1);
            assert_eq!(
                super::super::super::courts::native_head(&host).1,
                [before.0]
            );
        }
        if change {
            assert_eq!(super::super::super::courts::native_head(&host), before);
        }
        assert_eq!(
            request
                .query(query::model::Read("a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            1
        );
        let first = request
            .query(query::first_output::Read("first:a".into()))
            .execute();
        if seed {
            assert_eq!(first.unwrap().rows()[0].value, 10);
        } else {
            assert!(first.is_err());
        }
    }
}

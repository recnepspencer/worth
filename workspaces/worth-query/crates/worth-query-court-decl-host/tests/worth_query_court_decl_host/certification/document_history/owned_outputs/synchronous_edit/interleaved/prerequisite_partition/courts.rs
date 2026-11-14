use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

fn host() -> WorthQueryProgramApplicationRuntime<Schema, Program> {
    world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        1, None, world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() },
    )
}
fn sorted(mut partition: Partition) -> Partition {
    partition.ordinary.sort();
    partition.matching.sort();
    partition
}
#[test]
fn declared_absence_and_equality_partition_every_related_live_row() {
    let host = host();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let result = request.query(Read("a".into())).execute().unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(
        sorted(result.rows()[0].clone()),
        Partition {
            ordinary: vec!["m".into(), "partition:absent".into()],
            matching: vec!["partition:matching".into()],
        }
    );
    let empty = request.query(Read("occ:a".into())).execute().unwrap();
    assert_eq!(
        empty.rows()[0],
        Partition {
            ordinary: vec![],
            matching: vec![]
        }
    );
    assert!(
        request
            .query(Read("unavailable-model".into()))
            .execute()
            .is_err(),
        "an unavailable scope cannot masquerade as an absent discriminator"
    );
}

#[test]
fn excluded_candidate_field_changes_invalidate_the_complete_partition_source() {
    let host = host();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for value in [3, 1] {
        let original = request.query(Read("a".into())).execute().unwrap();
        let observed = original.observed_sources()[0].clone();
        let accepted = request
            .advanced()
            .mutate(expectation::Observe)
            .expect_source(observed.clone())
            .idempotency(&format!("partition-live-{value}"))
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                accepted,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{accepted:?}"
        );

        let mut edit = request
            .edit_envelope()
            .idempotency(format!("partition-change-{value}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<ApplicationNoOutputGraph, _>(fixture::calibration::Intent {
            key: "partition:excluded".into(),
            value,
            key_derived_target: false,
        })
        .unwrap();
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(outcome, WorthQueryEditEnvelopeOutcome::Committed(_)),
            "{outcome:?}"
        );
        let current = request.query(Read("a".into())).execute().unwrap();
        if value == 3 {
            assert_eq!(
                sorted(current.rows()[0].clone()),
                sorted(original.rows()[0].clone()),
                "false-to-false change preserves both selected sets"
            );
        } else {
            assert_eq!(
                sorted(current.rows()[0].clone()).matching,
                [
                    "partition:excluded".to_owned(),
                    "partition:matching".to_owned()
                ]
            );
        }
        let head = super::super::super::super::courts::native_head(&host);
        let denial = request
            .advanced()
            .mutate(expectation::Observe)
            .expect_source(observed)
            .idempotency(&format!("partition-stale-{value}"))
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_err();
        let WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        ) = denial
        else {
            panic!("{denial:?}")
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::SourceChanged
        );
        assert_eq!(super::super::super::super::courts::native_head(&host), head);
    }
}

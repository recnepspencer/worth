//! No-op and other-record effects cannot authenticate the preserved output's field change.
use super::*;

#[test]
fn unchanged_output_and_another_records_write_do_not_seal_zero_relation_changes() {
    for (case, fault) in [
        (Case::Unchanged, ObservationFault::None),
        (
            Case::Second {
                value: 42,
                required: 420,
            },
            ObservationFault::OtherRecordMutation,
        ),
    ] {
        let host = world::install_with_neighbour();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(Input {
                key: "record".into(),
                case: Case::First { value: 11 },
                link: LinkEdit::Set(LinkTarget::Named("neighbour".into())),
                observation_fault: ObservationFault::None,
            })
            .without_source()
            .idempotency(&"establish-nonempty".to_owned())
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
        let before = request.retain_current_read().unwrap();
        let outcome = request
            .advanced()
            .mutate(Input {
                key: "record".into(),
                case,
                link: LinkEdit::Unchanged,
                observation_fault: fault,
            })
            .without_source()
            .idempotency(&"no-output-field-change".to_owned())
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("{outcome:?}")
        };
        assert_eq!(
            denial
                .relation_preimage_failure()
                .expect("typed retention refusal")
                .kind(),
            WorthQueryRelationPreImageDenial::MissingMutation
        );
        assert_eq!(
            before.selected_commit(),
            request.retain_current_read().unwrap().selected_commit()
        );
        assert_eq!(
            world::read(&host),
            query::Row {
                class: 1,
                first: Some(11),
                second: None,
                required: None,
                third: None
            }
        );
        let neighbour = request
            .query(query::Read {
                key: "neighbour".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            neighbour.rows()[0],
            query::Row {
                class: 1,
                first: Some(99),
                second: None,
                required: None,
                third: None
            }
        );
        let links = request
            .query(link_query::Read {
                key: "record".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(links.rows()[0].targets, ["neighbour"]);
        assert_eq!(links.rows()[0].annotation, None);
    }
}

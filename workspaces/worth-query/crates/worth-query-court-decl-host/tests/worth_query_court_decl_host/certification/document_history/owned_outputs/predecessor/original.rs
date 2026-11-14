//! Original C includes own emitted output, unlike its Initial consumed INPUT.
use super::*;
worth_query_operation!(pub ReadOriginal for Schema, input query::ParametersBinding; access {
    read_entities: [Model, FirstOutput];
    read_fields: [ModelValue::at("ModelValue"), FirstKey::at("FirstKey"), FirstValue::at("FirstValue")];
    read_relations: [FirstOf];
});
worth_query_operation_requires!(ReadOriginal => [ReadInputs]);
pub const CONTRACT: WorthQueryProducerObservationReadContract<Schema> =
    WorthQueryProducerObservationReadContract::new(&[
        WorthQueryProducerObservationReadOperation::new(ReadOriginal::reference()),
    ]);
#[derive(Clone, Copy)]
pub(super) enum Mode {
    Observe,
    Undeclared,
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<ReadOriginal>(
            ReadOriginal::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_requires_ability(ReadOriginal::reference(), ReadInputs::reference())
}
pub(super) fn observe<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    source: &WorthQueryInvariantEntityIdentity<Schema, Model>,
) -> bool {
    let Some(mode) = PROBE.with(|probe| probe.borrow().original_mode) else {
        return false;
    };
    let current = reader
        .field(source, ModelValue::at("ModelValue"))
        .unwrap()
        .unwrap();
    let result = match mode {
        Mode::Undeclared => {
            reader.original_observation(UndeclaredRead::reference(), |_| (0, current, None))
        }
        Mode::Observe => reader.original_observation(ReadOriginal::reference(), |before| {
            let model = before.entity(Model::reference(), source).unwrap();
            let value = before
                .field(&model, ModelValue::at("ModelValue"))
                .unwrap()
                .unwrap();
            let rows = before.relations_to(FirstOf::reference(), &model).unwrap();
            let own = rows
                .iter()
                .find(|row| {
                    before
                        .field(row.from(), FirstKey::at("FirstKey"))
                        .unwrap()
                        .as_deref()
                        == Some("predecessor:a")
                })
                .map(|row| {
                    before
                        .field(row.from(), FirstValue::at("FirstValue"))
                        .unwrap()
                        .unwrap()
                });
            (value, current, own)
        }),
    };
    // Deliberately swallow refusal; the real producer owner must veto publication.
    PROBE.with(|probe| probe.borrow_mut().original_values.push(result));
    true
}
#[cfg(test)]
mod courts {
    use super::*;
    use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
    #[test]
    fn original_c_values_and_own_output_remain_distinct_from_applied_d() {
        let _observation = WorthQueryExecutionObservation::install_on_this_thread();
        let host = install(false);
        super::super::courts::reset(false);
        PROBE.with(|p| p.borrow_mut().original_mode = Some(Mode::Observe));
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        super::super::courts::settle(&request, &host).unwrap();
        assert_eq!(
            PROBE.with(|p| p.borrow().original_values.last().cloned()),
            Some(Ok((1, 1, None)))
        );
        for value in [2, 3] {
            let outcome = request
                .advanced()
                .mutate(synchronous::Intent {
                    key: "a".into(),
                    value,
                })
                .without_source()
                .idempotency(&format!("original-data-{value}"))
                .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(
                matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ),
                "{outcome:?}"
            );
            assert_eq!(
                PROBE.with(|p| p.borrow().original_values.last().cloned()),
                Some(Ok((value - 1, value, Some(value - 1 + 9000))))
            );
            assert_eq!(
                request
                    .query(query::first_output::Read("predecessor:a".into()))
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                value + 9000
            );
        }
    }
    #[test]
    fn original_read_permission_and_declaration_refusals_cannot_be_swallowed() {
        let _observation = WorthQueryExecutionObservation::install_on_this_thread();
        for revoke in [true, false] {
            let host = install(false);
            super::super::courts::reset(false);
            let scope = request_scope();
            let principal = authenticate_operator(host.installed_schema(), &scope);
            let request = host.request(&principal, &scope);
            super::super::courts::settle(&request, &host).unwrap();
            if revoke {
                let revoked = request
                    .advanced()
                    .mutate(member_edit::Intent {
                        key: "m".into(),
                        value: 6,
                    })
                    .without_source()
                    .idempotency(&"revoke-original-read".to_owned())
                    .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap();
                assert!(
                    matches!(
                        revoked,
                        WorthQueryApplicationMutationOutcome::Committed { .. }
                    ),
                    "{revoked:?}"
                );
            }
            PROBE.with(|p| {
                p.borrow_mut().original_mode = Some(if revoke {
                    Mode::Observe
                } else {
                    Mode::Undeclared
                })
            });
            let before = crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head(&host);
            let outcome = request
                .advanced()
                .mutate(synchronous::Intent {
                    key: "a".into(),
                    value: 2,
                })
                .without_source()
                .idempotency(&"refused-original-read".to_owned())
                .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(
                !matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ),
                "{outcome:?}"
            );
            PROBE.with(|p| {
                let p = p.borrow();
                let cause = p
                    .original_values
                    .last()
                    .expect("the attempted original read reached its owner");
                if revoke {
                    assert!(
                        matches!(
                            cause,
                            Err(WorthQueryProducerObservationReadDenial::Authorization(_))
                        ),
                        "{cause:?}"
                    );
                } else {
                    assert!(
                        matches!(
                            cause,
                            Err(WorthQueryProducerObservationReadDenial::UndeclaredObservationRead)
                        ),
                        "{cause:?}"
                    );
                }
            });
            assert_eq!(crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head(&host), before);
            assert_eq!(
                request
                    .query(query::first_output::Read("predecessor:a".into()))
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .value,
                9001
            );
        }
    }
}

use super::{fixture, *};
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryCompletedMutationCandidate,
};
use fixture::{QuotedBinding, QuotedInput, QuotedOperation, QuotedWriteOrder};
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};

fn execute(
    world: &ReservationWorld,
    input: &QuotedInput,
) -> Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<ReservationSchema, QuotedBinding>,
        QuotedInput,
    >,
    MutationHandlerExecutionDenial,
> {
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let principal = authenticated_principal(&phase, world, &request);
            let account = resolved_account(&phase, world, "open", &request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(QuotedOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(&phase)
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap();
            let key = "actual-installed-quote".to_owned();
            let identities =
                ApplicationMutationIdentities::<ReservationSchema, QuotedBinding>::encode(
                    &key, input,
                )
                .unwrap();
            world.application.execute_mutation_handler::<QuotedBinding>(
                &phase,
                &identities,
                &1,
                admission,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .unwrap()
}

fn assert_original_account_open(world: &ReservationWorld) {
    let fresh = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&phase, world, "open", &fresh);
        })
        .unwrap();
}

#[test]
fn installed_metadata_quote_funds_actual_create_fields_link_and_output_roles() {
    let world = installed_world();
    for (value, write_order, expected_writes) in [
        ("changed", QuotedWriteOrder::RequiredOnly, 3),
        (
            "a second actual source payload with a different retained width",
            QuotedWriteOrder::RequiredThenOptional,
            4,
        ),
        (
            "optional-first-value",
            QuotedWriteOrder::OptionalThenRequiredAndReplacement,
            5,
        ),
        (
            "cleared-then-set",
            QuotedWriteOrder::OptionalClearThenSet,
            5,
        ),
        ("cleared-twice", QuotedWriteOrder::OptionalClearThenClear, 5),
    ] {
        let input = QuotedInput {
            value: value.to_owned(),
            underquote: false,
            write_order,
        };
        let completed = match execute(&world, &input).unwrap() {
            HandlerResult::Completed(completed) => completed,
            _ => panic!("the original installed metadata quote must fit its actual writer"),
        };
        let (program, returned) = completed.into_parts();
        assert_eq!(returned, input);
        assert_eq!(
            program.effects.len(),
            3,
            "create, update, link; initialization stays on create"
        );
        assert_eq!(
            program
                .candidate_requirements
                .unwrap()
                .cardinality()
                .maximum_writes(),
            expected_writes,
        );
        if write_order == QuotedWriteOrder::OptionalThenRequiredAndReplacement {
            let expected =
                StringApplicationValueBinding::encode(&format!("{value}-replacement")).unwrap();
            let patch = program.effects.iter().find_map(|effect| match effect {
                super::super::WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. } => Some(fields),
                _ => None,
            }).expect("the actual optional-first update is a retained patch");
            assert_eq!(
                patch.len(),
                2,
                "optional and required entries share this actual target"
            );
            assert!(
                patch
                    .values()
                    .any(|write| write.value.as_ref() == Some(&expected)),
                "the second write replaced the same required entry's retained value"
            );
        }
        if matches!(
            write_order,
            QuotedWriteOrder::OptionalClearThenSet | QuotedWriteOrder::OptionalClearThenClear
        ) {
            let patch = program.effects.iter().find_map(|effect| match effect {
                super::super::WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. } => Some(fields),
                _ => None,
            }).expect("reusing a cleared optional slot retains the same actual patch");
            assert_eq!(patch.len(), 2);
            assert_eq!(
                patch.values().filter(|write| write.value.is_none()).count(),
                usize::from(write_order == QuotedWriteOrder::OptionalClearThenClear)
            );
            let expected = StringApplicationValueBinding::encode(&value.to_owned()).unwrap();
            assert!(
                patch
                    .values()
                    .filter_map(|write| write.value.as_ref())
                    .all(|value| value == &expected),
                "the actual retained set value and required payload match their original input"
            );
        }
        drop(program);
        assert_original_account_open(&world);
    }
}

#[test]
fn installed_metadata_quote_one_byte_short_refuses_exact_representation_debit() {
    let world = installed_world();
    for write_order in [
        QuotedWriteOrder::RequiredThenOptional,
        QuotedWriteOrder::OptionalThenRequiredAndReplacement,
        QuotedWriteOrder::OptionalClearThenSet,
        QuotedWriteOrder::OptionalClearThenClear,
    ] {
        let input = QuotedInput {
            value: "candidate-only-value".to_owned(),
            underquote: true,
            write_order,
        };
        let refusal = match execute(&world, &input) {
        Err(MutationHandlerExecutionDenial::Handler(cause)) => cause
            .downcast::<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>().unwrap(),
        _ => panic!("one-byte underestimate must reach the actual writer's representation debit"),
        };
        assert_eq!(
            refusal.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
        );
        assert_original_account_open(&world);
    }
}

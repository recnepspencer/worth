use super::{fixture, *};
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryCompletedMutationCandidate,
};
use crate::domain_computation::primary_graph::tests::fixture::RetainedStatusNotice;
use fixture::{
    QuotedEmissionBinding, QuotedEmissionInput, QuotedEmissionOperation, QuotedRetainedEffect,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;

fn execute(
    world: &ReservationWorld,
    input: &QuotedEmissionInput,
) -> Result<
    HandlerResult<
        WorthQueryCompletedMutationCandidate<ReservationSchema, QuotedEmissionBinding>,
        QuotedEmissionInput,
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
                .installed_operation(QuotedEmissionOperation::reference())
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
            let key = "actual-emission-quote".to_owned();
            let identities = ApplicationMutationIdentities::<
                ReservationSchema,
                QuotedEmissionBinding,
            >::encode(&key, input)
            .unwrap();
            world
                .application
                .execute_mutation_handler::<QuotedEmissionBinding>(
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
fn installed_emission_quote_funds_original_typed_payload_debit() {
    let world = installed_world();
    for value in [
        "published",
        "a wider actual retained source publication payload",
    ] {
        let input = QuotedEmissionInput {
            value: value.to_owned(),
            underquote: false,
        };
        let completed = match execute(&world, &input).unwrap() {
            HandlerResult::Completed(completed) => completed,
            _ => panic!("measured ordinary emission must fit the original writer's debit"),
        };
        let (program, returned) = completed.into_parts();
        assert_eq!(returned, input);
        let [super::super::WorthQueryApplicationRealizedEffect::Emit(emission)] =
            program.effects.as_slice()
        else {
            panic!("the installed handler must retain exactly its actual typed emission");
        };
        assert_eq!(emission.effect(), QuotedRetainedEffect::reference().name());
        assert_eq!(
            emission.payload::<RetainedStatusNotice>(),
            Some(&RetainedStatusNotice(value.to_owned()))
        );
        assert!(
            emission.is_well_formed(),
            "original payload identity and measurement survive custody"
        );
        drop(program);
        assert_original_account_open(&world);
    }
}

#[test]
fn installed_emission_quote_one_byte_short_reaches_writer_reservation_refusal() {
    let world = installed_world();
    let input = QuotedEmissionInput {
        value: "published".to_owned(),
        underquote: true,
    };
    let refusal = match execute(&world, &input) {
        Err(MutationHandlerExecutionDenial::Handler(cause)) => cause
            .downcast::<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>().unwrap(),
        _ => panic!("one-byte underestimate must reach the real emission reservation debit"),
    };
    assert_eq!(
        refusal.kind(),
        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
    );
    assert_original_account_open(&world);
}

#[test]
fn installed_emission_quote_preserves_original_payload_validation() {
    let world = installed_world();
    let input = QuotedEmissionInput {
        value: String::new(),
        underquote: false,
    };
    let refusal = match execute(&world, &input) {
        Err(MutationHandlerExecutionDenial::Handler(cause)) => cause
            .downcast::<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>().unwrap(),
        _ => panic!("the original notice binding rejects an empty payload during quote"),
    };
    assert_eq!(
        refusal.kind(),
        WorthQueryApplicationAttemptDenialKind::InvalidEffectValue
    );
    assert_original_account_open(&world);
}

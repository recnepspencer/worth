//! Original result-set conditions refuse natively before any candidate effect.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationIntent,
    application_schema::{ApplicationEncodedScalarValue, TypedMutationPreconditions},
};
use worth_query_host::facade::application_entry::WorthQueryOperationAdvance as Advance;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, AdjustPlanarSource,
    Body, BodyKey, PlanarMutationScope, PlanarSourceAdjustmentBinding,
};

type Conditions = TypedMutationPreconditions<ConsumerSchema, AdjustPlanarSource, Body>;

fn conditions(expected: &str) -> Conditions {
    TypedMutationPreconditions::new().expect_fact(
        BodyKey::reference::<ConsumerSchema>(),
        ApplicationEncodedScalarValue::try_new(expected.to_owned()).unwrap(),
    )
}

// Domain input is retained once; the owning intent itself cannot be cloned.
struct OriginalAdjustment(PlanarSourceAdjustment);
impl ApplicationMutationIntent<ConsumerSchema> for OriginalAdjustment {
    type Binding = PlanarSourceAdjustmentBinding<ConsumerSchema>;
    fn input(&self) -> &PlanarSourceAdjustment {
        &self.0
    }
    fn scope_binding(&self) -> PlanarMutationScope<ConsumerSchema> {
        <PlanarSourceAdjustment as ApplicationMutationIntent<ConsumerSchema>>::scope_binding(
            &self.0,
        )
    }
}

#[test]
fn owning_result_set_false_original_condition_refuses_before_candidate_effect() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(7),
    };
    reset_planar_source_adjustment_contacts(&input);
    let original = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let operation = worth_query_topology_entry::capture_original(
        &request,
        OriginalAdjustment(input.clone()),
        conditions("anchor-b"),
        original.result_set_observation().unwrap().clone(),
        10_077,
    )
    .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
    .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
    .unwrap_or_else(|(_, denial)| {
        panic!("native precondition is checked after construction: {denial:?}")
    });
    let mut operation = operation;
    for _ in 0..2 {
        operation = match operation.advance(&request) {
            Advance::Refused { operation, denial } => {
                use worth_query_host::facade::{
                    application_entry::{
                        WorthQueryApplicationRequestMutationDenial,
                        WorthQueryOperationAdvanceDenial,
                        WorthQueryPerformedMutationExecutionDenial,
                    },
                    primary_graph::{
                        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenialKind,
                    },
                };
                assert!(
                    matches!(denial,
                        WorthQueryOperationAdvanceDenial::Mutation(WorthQueryPerformedMutationExecutionDenial::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::Attempt(ref cause))))
                        if cause.kind() == WorthQueryApplicationAttemptDenialKind::MutationPreconditionMismatch
                    ),
                    "actual original fact refusal: {denial:?}"
                );
                operation
            }
            _ => panic!("a false original fact must retain refused initial custody"),
        };
    }
    // Decision resolves the target; the mismatch stops before candidate/effects.
    assert_eq!(planar_source_adjustment_contacts(&input), (2, 0));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(1)
    );
    // Refused initial custody has issued no native recovery or output obligation.
    drop(operation);
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
}

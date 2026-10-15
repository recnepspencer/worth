//! Complete original result-set meaning crosses the ordinary fixed lifecycle.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_decl::facade::{
    application_operation::ApplicationMutationIntent,
    application_schema::{ApplicationEncodedScalarValue, TypedMutationPreconditions},
};
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationProgress as Progress,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, AdjustPlanarSource,
    Body, BodyKey, PlanarMutationScope, PlanarSourceAdjustmentBinding,
};

mod hostile;

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
fn owning_result_set_original_conditions_survive_partial_publication() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(7),
    };
    reset_planar_source_adjustment_contacts(&input);
    let operation = {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let observed = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap();
        worth_query_topology_entry::capture_original(
            &request,
            OriginalAdjustment(input.clone()),
            conditions("anchor-a"),
            observed.result_set_observation().unwrap().clone(),
            10_076,
        )
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("installed original descriptor: {denial:?}"))
    };
    // No initiating principal, scope or query observation remains live.
    world
        .application
        .with_host_advancement(|phase| {
            world.application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let operation = match operation.advance(&world.application.request(&principal, &scope)) {
        Advance::Progress {
            operation,
            progress: Progress::SourceUnpublished,
        } => operation,
        _ => panic!("real append failure must retain the original result-set operation"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert_eq!(
        world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );
    drop(principal);
    drop(scope);

    let mut operation = Some(operation);
    let mut recovered = false;
    let mut promoted = false;
    let settled = (0..32).find_map(|_| {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(), &scope,
        )).unwrap();
        let request = world.application.request(&principal, &scope);
        match operation.take().unwrap().advance(&request) {
            Advance::Progress { operation: next, progress } => {
                recovered |= matches!(progress, Progress::Recovery(worth_query_host::facade::application_entry::WorthQueryProgramSourceRecoveryProgress::Performed));
                promoted |= matches!(progress, Progress::SourcePromoted);
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => panic!("original result-set progression: {denial:?}"),
        }
    }).expect("the authored fixed demand must settle");
    assert!(recovered && promoted);
    assert!(settled.handler_result().is_none());
    assert!(settled.recovered_progress().is_some());
    assert_eq!(settled.settlement().output_count(), 4);
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        1
    );
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let source_truth = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(source_truth.rows().len(), 1);
    assert_eq!(source_truth.rows()[0].body_key, "anchor-a");
    assert_eq!(source_truth.rows()[0].y, length(7));
    // Independent product observation: the authored selection did not mutate its sibling.
    let sibling_truth = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(sibling_truth.rows().len(), 1);
    assert_eq!(sibling_truth.rows()[0].body_key, "anchor-b");
    assert_eq!(sibling_truth.rows()[0].y, length(1));
    drop(sibling_truth);
    drop(source_truth);
    let delivered = settled
        .read_current(&request)
        .expect("declared delivery uses current settlement");
    assert_eq!(delivered.rows()[0].value, length(8));
    drop(delivered);
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful original cleanup: {denial:?}"));
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
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

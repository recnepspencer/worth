//! Host-selected staging limits govern the same ordinary installed mutation.
use std::num::{NonZeroU64, NonZeroUsize};

use super::amendment;
use super::product_workflow_support::{
    self, principal as authenticate, read_input, ExampleApplication,
};
use product_workflow_support::adapters::ClockSource;
use product_workflow_support::application::example_limits;
use product_workflow_support::conditional_contribution::{
    TemporalConditional, TemporalContributionConfiguration,
};
use product_workflow_support::{program, schema::TemporalHostSchema};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
    },
    application_installation::{
        self, WorthQueryApplicationCheckpoint, WorthQueryTransactionStagingResources as Resources,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted,
        WorthQueryInvariantExecutionDenialKind as Kind,
        WorthQueryInvariantExecutionFailurePosture as Posture, WorthQueryMutationHandlerWork,
    },
};

fn resources(bytes: u64, loci: usize) -> Resources {
    Resources::bounded(
        NonZeroU64::new(bytes).unwrap(),
        NonZeroUsize::new(loci).unwrap(),
    )
}

#[derive(Debug)]
enum ExpectedBoundary {
    Overlay,
    Footprint,
}

fn restore(checkpoint: WorthQueryApplicationCheckpoint, staging: Resources) -> ExampleApplication {
    let (clock_source, clock_control) = ClockSource::due();
    let runtime = application_installation::in_memory_program_from_checkpoint(
        program::validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration { clock_source },),
        example_limits().with_transaction_staging_resources(staging),
        checkpoint,
    )
    .expect("the same schema/program checkpoint admits the selected staging resources");
    assert!(runtime.checkpoint_restore_work().is_some());
    let conditional = runtime.conditional::<TemporalConditional>().unwrap();
    ExampleApplication {
        runtime,
        conditional,
        clock_control,
    }
}

#[test]
fn installed_mutation_staging_limits_refuse_without_effect_and_reselect_on_restore() {
    let mut seeded = ExampleApplication::publish("blocked");
    let checkpoint = seeded.runtime.capture_application_checkpoint().unwrap();
    seeded.runtime.close_conditional_runtime().unwrap();
    drop(seeded);

    // Both envelopes are positive and finite. Each isolates one real staging
    // boundary, without narrowing candidate or decision admission.
    for (expected, staging) in [
        (ExpectedBoundary::Overlay, resources(1, 128)),
        (ExpectedBoundary::Footprint, resources(1_048_576, 1)),
    ] {
        let mut app = restore(checkpoint.clone(), staging);
        let scope = product_workflow_support::adapters::request_scope();
        let principal = authenticate(&app, &scope);
        let branch = app.runtime.current_world();
        let request = app.runtime.request(&principal, &scope);
        let before = request.retain_read().unwrap();
        let input = read_input(&app, branch, &principal, &scope);
        let key = 0x85_u64;
        let (outcome, work) = request
            .mutate(amendment("staged", 2))
            .without_source()
            .idempotency(&key)
            .execute_in_program_report(&app.runtime)
            .into_parts();
        let WorthQueryMutationHandlerWork::Captured(capture) = work else {
            panic!("staging refusal must follow the installed decision: {outcome:?}");
        };
        assert!(capture.handler_contacted());
        let outcome = outcome.unwrap();
        assert!(outcome.result().is_none());
        assert!(outcome.receipt().is_none());
        let Outcome::Commit(Uncommitted::Denied(denial)) = outcome else {
            panic!("the selected staging boundary must refuse: {outcome:?}");
        };
        let failure = denial.invariant_execution_failure().unwrap();
        assert_eq!(failure.posture(), Posture::Exhausted);
        match (&expected, failure.kind()) {
            (
                ExpectedBoundary::Overlay,
                Kind::TransactionOverlayCapacityExhausted {
                    maximum_bytes,
                    required_bytes,
                },
            ) => {
                assert_eq!(staging.maximum_overlay_bytes(), 1);
                assert_eq!(maximum_bytes, 1);
                assert!(required_bytes > maximum_bytes);
            }
            (
                ExpectedBoundary::Footprint,
                Kind::TransactionFootprintCapacityExhausted {
                    maximum_loci,
                    required_loci,
                },
            ) => {
                assert_eq!(staging.maximum_footprint_loci(), 1);
                assert_eq!(maximum_loci, 1);
                assert!(required_loci > maximum_loci);
            }
            other => panic!("staging must refuse at the intended independent boundary: {other:?}"),
        }
        let after = request.retain_read().unwrap();
        assert_eq!(before.selected_commit(), after.selected_commit());
        assert_eq!(read_input(&app, branch, &principal, &scope), input);
        drop((before, after, request));
        let refused_checkpoint = app.runtime.capture_application_checkpoint().unwrap();
        app.runtime.close_conditional_runtime().unwrap();
        drop(app);

        let mut adequate = restore(refused_checkpoint, resources(1_048_576, 128));
        let principal = authenticate(&adequate, &scope);
        let outcome = adequate
            .runtime
            .request(&principal, &scope)
            .mutate(amendment("staged", 2))
            .without_source()
            .idempotency(&key)
            .execute_in_program(&adequate.runtime)
            .unwrap();
        assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
        assert_eq!(outcome.result().unwrap().revision, 2);
        assert_eq!(
            read_input(
                &adequate,
                adequate.runtime.current_world(),
                &principal,
                &scope
            ),
            "staged"
        );
        let committed_checkpoint = adequate.runtime.capture_application_checkpoint().unwrap();
        adequate.runtime.close_conditional_runtime().unwrap();
        drop(adequate);

        let mut reopened = restore(committed_checkpoint, resources(2_097_152, 256));
        let principal = authenticate(&reopened, &scope);
        assert_eq!(
            read_input(
                &reopened,
                reopened.runtime.current_world(),
                &principal,
                &scope
            ),
            "staged"
        );
        let outcome = reopened
            .runtime
            .request(&principal, &scope)
            .mutate(amendment("restored", 3))
            .without_source()
            .idempotency(&0x86_u64)
            .execute_in_program(&reopened.runtime)
            .unwrap();
        assert!(matches!(outcome, Outcome::Committed { .. }), "{outcome:?}");
        assert_eq!(outcome.result().unwrap().revision, 3);
        assert_eq!(
            read_input(
                &reopened,
                reopened.runtime.current_world(),
                &principal,
                &scope
            ),
            "restored"
        );
        reopened.runtime.close_conditional_runtime().unwrap();
    }
}

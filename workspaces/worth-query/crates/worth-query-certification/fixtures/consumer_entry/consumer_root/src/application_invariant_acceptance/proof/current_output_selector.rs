use std::error::Error;
use std::num::NonZeroUsize;

use worth_query_consumer_values::{
    PlanarCurrentOutputExpectation, PlanarMutationDenial, PlanarOperation,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationProgramOutputProgress,
        WorthQueryApplicationRequestMutationDenial, WorthQueryOutputDemandControls,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryCurrentOutputDenial,
        WorthQueryCurrentOutputDenialKind,
    },
};
use worth_query_topology_entry::{PlanarEdit, PlanarMutation};
use worth_query_topology_entry::{PlanarOutputDemand, PlanarSourceAdjustment};

use super::{length, output_correspondence::observed_source, read_y, ProgramApplication, Request};

pub(super) fn producer_qualified_missing_and_stale(
    request: &Request<'_>,
    application: &ProgramApplication,
) {
    publish_initial(request, application, "anchor-a");
    publish_initial(request, application, "sibling-a");

    let outcome = execute_verification(
        request,
        application,
        &[expectation("anchor-a"), expectation("sibling-a")],
        962,
    )
    .expect("two producer selections reach the installed handler");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "a producer-qualified cache must not alias two same-family producers: {outcome:?}"
    );

    let missing = execute_verification(request, application, &[expectation("anchor-b")], 963)
        .expect("missing output is a semantic selector outcome");
    assert!(matches!(
        missing,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            PlanarMutationDenial::CurrentOutputMissing
        )
    ));

    publish_initial(request, application, "anchor-a");
    adjust_source(
        request,
        application,
        "anchor-a",
        read_y(request, "anchor-a") + 1,
        965,
    );
    let before = request
        .query(worth_query_topology_entry::PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .receipt()
        .inspect()
        .basis()
        .version();
    let denial = execute_verification(request, application, &[expectation("anchor-a")], 966)
        .expect_err("a changed retained source fact must reject the old output");
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(handler),
    ) = denial
    else {
        panic!("stale output must retain its handler execution family: {denial:?}")
    };
    let current_output = handler
        .source()
        .and_then(|source| source.downcast_ref::<WorthQueryCurrentOutputDenial>())
        .expect("the handler denial retains the typed current-output cause");
    assert_eq!(
        current_output.kind(),
        WorthQueryCurrentOutputDenialKind::StaleSource
    );
    let after = request
        .query(worth_query_topology_entry::PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .receipt()
        .inspect()
        .basis()
        .version();
    assert_eq!(before, after, "stale selection must publish nothing");
}

fn publish_initial(request: &Request<'_>, application: &ProgramApplication, scope_key: &str) {
    let mut output = request
        .start_program_outputs(
            application,
            PlanarOutputDemand::new(scope_key),
            WorthQueryOutputDemandControls::new(
            NonZeroUsize::new(4_096).unwrap(),
            NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .expect("the declared root producer starts for the selected source");
    loop {
        match output
            .advance(request)
            .expect("the declared root producer settles")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
        }
    }
}

fn adjust_source(
    request: &Request<'_>,
    application: &ProgramApplication,
    scope_key: &str,
    y: u64,
    command: u64,
) {
    let source = observed_source(request, scope_key);
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: scope_key.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(source)
        .idempotency(&command)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(application)
        .expect("the independent source adjustment reaches its real mutation owner");
    assert!(matches!(
        outcome,
        worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)
    ));
}

fn execute_verification(
    request: &Request<'_>,
    application: &ProgramApplication,
    expectations: &[PlanarCurrentOutputExpectation],
    command: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<
        worth_query_consumer_values::PlanarMutationDenial,
        worth_query_consumer_values::PlanarAdjustmentResult,
    >,
    WorthQueryApplicationRequestMutationDenial,
> {
    let input = PlanarMutation {
        scope_key: "anchor-a".to_owned(),
        operation: PlanarOperation::VerifyCurrentOutputs(expectations.to_vec()),
        validator_work: 4096,
    };
    request
        .mutate(PlanarEdit(input))
        .expect_source(observed_source(request, "anchor-a"))
        .idempotency(&command)
        .execute_in_program(application)
}

fn expectation(key: &str) -> PlanarCurrentOutputExpectation {
    PlanarCurrentOutputExpectation {
        producer_key: key.to_owned(),
        output_key: key.to_owned(),
    }
}

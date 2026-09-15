use std::error::Error;

use worth_query_consumer_values::{
    PlanarAdjustment, PlanarCurrentOutputExpectation, PlanarMutationDenial, PlanarOperation,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryCurrentOutputDenial,
        WorthQueryCurrentOutputDenialKind,
    },
};
use worth_query_topology_entry::AlternatePlanarOutput;
use worth_query_topology_entry::PlanarMutation;
use worth_query_topology_entry::PlanarSourceAdjustment;

use super::{length, output_correspondence::observed_source, read_y, Request};

pub(super) fn producer_qualified_selection_is_current(
    request: &Request<'_>,
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        crate::ConsumerSchema,
        crate::ConsumerProgram,
    >,
) {
    publish_no_change(request, "anchor-a", &["anchor-a"], 960);
    publish_no_change(request, "sibling-a", &["sibling-a"], 961);

    let outcome = execute_verification(
        request,
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

    let missing = execute_verification(request, &[expectation("anchor-b")], 963)
        .expect("missing output is a semantic selector outcome");
    assert!(matches!(
        missing,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            PlanarMutationDenial::CurrentOutputMissing
        )
    ));

    publish_no_change(request, "anchor-b", &["anchor-b"], 967);
    publish_alternate(request, "anchor-b", "anchor-b", 968);
    let duplicate = execute_verification(request, &[expectation("anchor-b")], 969)
        .expect("duplicate correspondence targets reach the installed handler");
    assert!(
        matches!(
            duplicate,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "two correspondences for one entity remain one unique output: {duplicate:?}"
    );

    publish_alternate(request, "anchor-b", "sibling-b", 970);
    let ambiguous = execute_verification(request, &[expectation("anchor-b")], 971)
        .expect("ambiguous output selection is a semantic handler outcome");
    assert!(matches!(
        ambiguous,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            PlanarMutationDenial::CurrentOutputAmbiguous
        )
    ));

    publish_no_change(request, "anchor-a", &["anchor-a"], 964);
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
    let denial = execute_verification(request, &[expectation("anchor-a")], 966)
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

fn publish_alternate(request: &Request<'_>, scope_key: &str, output_key: &str, command: u64) {
    let source = observed_source(request, scope_key);
    let outcome = request
        .mutate(AlternatePlanarOutput {
            scope_key: scope_key.to_owned(),
            output_key: output_key.to_owned(),
        })
        .expect_source(source)
        .idempotency(&command)
        .execute()
        .expect("the alternate output reaches its real public mutation owner");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "alternate output must commit: {outcome:?}"
    );
}

fn publish_no_change(request: &Request<'_>, scope_key: &str, observed_keys: &[&str], command: u64) {
    let adjustments = observed_keys
        .iter()
        .map(|key| PlanarAdjustment {
            body_key: (*key).to_owned(),
            replacement_y: length(read_y(request, key)),
        })
        .collect();
    publish(request, scope_key, adjustments, command);
}

fn adjust_source(
    request: &Request<'_>,
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        crate::ConsumerSchema,
        crate::ConsumerProgram,
    >,
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
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramInventory, crate::ConsumerProgramRoot>(application)
        .expect("the independent source adjustment reaches its real mutation owner");
    assert!(matches!(
        outcome,
        worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)
    ));
}

fn publish(
    request: &Request<'_>,
    scope_key: &str,
    adjustments: Vec<PlanarAdjustment>,
    command: u64,
) {
    let input = PlanarMutation {
        scope_key: scope_key.to_owned(),
        operation: PlanarOperation::Adjust(adjustments),
        validator_work: 4096,
    };
    let source = observed_source(request, scope_key);
    let outcome = request
        .mutate(input)
        .expect_source(source)
        .idempotency(&command)
        .execute()
        .expect("the output setup reaches its real mutation owner");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

fn execute_verification(
    request: &Request<'_>,
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
        .mutate(input)
        .expect_source(observed_source(request, "anchor-a"))
        .idempotency(&command)
        .execute()
}

fn expectation(key: &str) -> PlanarCurrentOutputExpectation {
    PlanarCurrentOutputExpectation {
        producer_key: key.to_owned(),
        output_key: key.to_owned(),
    }
}

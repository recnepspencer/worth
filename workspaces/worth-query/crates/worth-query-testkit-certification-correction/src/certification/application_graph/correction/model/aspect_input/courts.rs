//! Real optional-aspect edits, original Native absence revisions and actual producers.
use super::super::causal_inspection::{independent::retained_query, journey, output_query};
use super::super::{
    aspect_input::{
        authoring::{Action, AspectEdit, Disclosure as AspectDisclosure},
        program::Root,
    },
    authentication::*,
    host,
    output_program::OutputProgram,
    schema::*,
};
#[path = "courts/source_affinity.rs"]
mod source_affinity;
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
use worth_query_host::facade::{application_entry::*, causal_inspection::*, primary_graph::*};

pub fn whole_aspect_removal_and_recreation_keep_the_particular_source() {
    let application = host::install_with_outputs();

    let mut earlier_absence = None;
    for (ordinal, aspect, expected) in [
        (
            0,
            Action::Set {
                bias: 4,
                sibling: 40,
            },
            15,
        ),
        (1, Action::Remove, 11),
        (
            2,
            Action::Set {
                bias: 7,
                sibling: 70,
            },
            18,
        ),
        (3, Action::Remove, 11),
    ] {
        let paired_before = (ordinal == 3).then(|| source_affinity::observe(&application));
        let (source, producer, read) = perform(
            &application,
            aspect,
            &format!("optional-aspect-{ordinal}"),
            1,
        );
        let producer = producer.expect("a changed optional aspect actually runs its producer");
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let (after, row) = retained_query::read(&application, &principal, &scope, &read);
        assert_eq!(row.calculated, expected);
        drop(read);
        assert_connection(&application, &source, &producer, &after);
        if ordinal == 1 {
            earlier_absence = Some(source);
        } else if ordinal == 3 {
            let old = earlier_absence.take().unwrap();
            assert_ne!(old.commit_reference(), source.commit_reference());
            let fresh = application.runtime().request(&principal, &scope);
            let denial = fresh
                .inspect::<output_query::Binding>(&after)
                .why_observed()
                .caused_by::<AspectDisclosure>(&old)
                .reference_only()
                .execute_governed(
                    journey::access(&application, &principal, &scope),
                    journey::access(&application, &principal, &scope),
                    journey::access(&application, &principal, &scope),
                    journey::access(&application, &principal, &scope),
                )
                .err()
                .expect("an earlier absent occurrence cannot replace this clear");
            assert_eq!(
                denial.producer_connection_refusal(),
                Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
            );
        }
        if let Some((before, row)) = paired_before {
            assert_eq!(
                (row.bias, row.sibling, row.calculated),
                (Some(7), Some(70), 18)
            );
            source_affinity::refuse_foreign_host(&application, &after);
            source_affinity::refuse_sibling_source(&application, &before);
            before.close();
        }
        after.close();
    }
}

pub fn no_op_clear_is_not_the_source_that_created_the_absent_revision() {
    let application = host::install_with_outputs();
    let _ = perform(
        &application,
        Action::Set {
            bias: 3,
            sibling: 30,
        },
        "aspect-present",
        1,
    );
    let (clear, producer, read) = perform(&application, Action::Remove, "aspect-remove", 1);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (after, row) = retained_query::read(&application, &principal, &scope, &read);
    assert_eq!(row.calculated, 11);
    drop(read);
    assert_connection(&application, &clear, &producer.unwrap(), &after);
    // Native semantic equality keeps the old absent revision. This demand reuses;
    // its zero contacts are deliberately not called performed output suppression.
    let (noop, _, reused_read) = perform(&application, Action::Remove, "aspect-remove-again", 0);
    drop(reused_read);
    assert_ne!(noop.commit_reference(), clear.commit_reference());
    let fresh = application.runtime().request(&principal, &scope);
    let denied = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<AspectDisclosure>(&noop)
        .reference_only()
        .execute_governed(
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
        )
        .err()
        .expect("a no-op clear cannot mint another absence revision");
    assert_eq!(
        denied.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
    );
    after.close();
}

fn assert_connection(
    application: &host::OutputHost,
    source: &WorthQueryApplicationCommitReceipt,
    producer: &WorthQueryApplicationCommitReceipt,
    after: &WorthQueryApplicationQueryObservation,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let fresh = application.runtime().request(&principal, &scope);
    let report = fresh
        .inspect::<output_query::Binding>(after)
        .why_observed()
        .caused_by::<AspectDisclosure>(source)
        .reference_only()
        .execute_governed(
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
        )
        .unwrap();
    assert_eq!(
        report.producer_connection().source_commit(),
        source.commit_reference()
    );
    assert_eq!(
        report.producer_connection().producer_commit(),
        producer.commit_reference()
    );
    assert_eq!(
        report.producer_connection().output_role(),
        "aspect-calculated-number"
    );
    assert_eq!(
        readers.observe().active(),
        baseline,
        "no retained Native reader"
    );
    report.close();
    assert_eq!(readers.observe().active(), baseline);
}

pub(super) fn perform(
    application: &host::OutputHost,
    action: Action,
    key: &str,
    expected_contacts: usize,
) -> (
    WorthQueryApplicationCommitReceipt,
    Option<WorthQueryApplicationCommitReceipt>,
    WorthQueryApplicationReadObservation,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let key = key.to_owned();
    let source = request
        .advanced()
        .mutate(AspectEdit {
            key: "numeric".to_owned(),
            action,
        })
        .without_source()
        .idempotency(&key)
        .execute_performed::<OutputProgram, Root>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let source = match source {
        WorthQueryApplicationPerformedMutationOutcome::Performed(source) => source,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("ordinary aspect source must perform; actual native outcome: {outcome:?}")
        }
        _ => panic!("ordinary aspect source must perform; required-output custody was not ready"),
    };
    let receipt = source.receipt().clone();
    let mut started = source
        .start_required_outputs(
            application,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|failure| panic!("actual required output starts: {:?}", failure.denial()));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(application, &request)
        .unwrap()
    else {
        panic!("actual aspect producer settles")
    };
    assert_eq!(
        settled.root_producer_contacts_in_this_demand(),
        expected_contacts
    );
    (
        receipt,
        settled.root_receipt().cloned(),
        settled.root_observation().retain(),
    )
}

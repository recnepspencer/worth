//! Real optional-operand edits, original Native absence revisions and actual producers.
use super::super::{
    authentication::*,
    host,
    operand_edit::{Disclosure as OperandDisclosure, OperandEdit},
    output_program::{OutputProgram, Root},
    schema::*,
};
use super::{independent::retained_query, journey, output_query};
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
use worth_query_host::facade::{application_entry::*, causal_inspection::*, primary_graph::*};

pub fn present_clear_present_and_absence_aba_keep_the_particular_source() {
    let application = host::install_with_outputs();
    assert_program_delivery_is_required(&application);
    let mut earlier_absence = None;
    for (ordinal, operand, expected) in [
        (0, Some(4), 15),
        (1, None, 11),
        (2, Some(7), 18),
        (3, None, 11),
    ] {
        let (source, producer, read) = perform(
            &application,
            operand,
            &format!("optional-input-{ordinal}"),
            1,
        );
        let producer = producer.expect("a changed optional operand actually runs its producer");
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let (after, row) = retained_query::read(&application, &principal, &scope, &read);
        assert_eq!(row.calculated, expected);
        drop(read);
        assert_connection(&application, &source, &producer, &after);
        if ordinal == 0 {
            assert_primary_disclosure_refuses_operand_source(&application, &source, &after);
        }
        if ordinal == 1 {
            earlier_absence = Some(source);
        } else if ordinal == 3 {
            let old = earlier_absence.take().unwrap();
            assert_ne!(old.commit_reference(), source.commit_reference());
            let fresh = application.runtime().request(&principal, &scope);
            let denial = fresh
                .inspect::<output_query::Binding>(&after)
                .why_observed()
                .caused_by::<OperandDisclosure>(&old)
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
        after.close();
    }
}

pub fn no_op_clear_is_not_the_source_that_created_the_absent_revision() {
    let application = host::install_with_outputs();
    let _ = perform(&application, Some(3), "operand-present", 1);
    let (clear, producer, read) = perform(&application, None, "operand-clear", 1);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (after, row) = retained_query::read(&application, &principal, &scope, &read);
    assert_eq!(row.calculated, 11);
    drop(read);
    assert_connection(&application, &clear, &producer.unwrap(), &after);
    // Native semantic equality keeps the old absent revision. This demand reuses;
    // its zero contacts are deliberately not called performed output suppression.
    let (noop, _, reused_read) = perform(&application, None, "operand-clear-again", 0);
    drop(reused_read);
    assert_ne!(noop.commit_reference(), clear.commit_reference());
    let fresh = application.runtime().request(&principal, &scope);
    let denied = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<OperandDisclosure>(&noop)
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
        .caused_by::<OperandDisclosure>(source)
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
        "calculated-number"
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
    value: Option<u64>,
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
        .mutate(OperandEdit {
            key: "numeric".to_owned(),
            value,
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
            panic!("ordinary operand source must perform; actual native outcome: {outcome:?}")
        }
        _ => panic!("ordinary operand source must perform; required-output custody was not ready"),
    };
    journey::settle_source(application, &request, source, expected_contacts)
}

fn assert_primary_disclosure_refuses_operand_source(
    application: &host::OutputHost,
    source: &WorthQueryApplicationCommitReceipt,
    after: &WorthQueryApplicationQueryObservation,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let fresh = application.runtime().request(&principal, &scope);
    let denial = fresh
        .inspect::<output_query::Binding>(after)
        .why_observed()
        .caused_by::<super::SourceDisclosure>(source)
        .reference_only()
        .execute_governed(
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
        )
        .err()
        .expect("Primary's genuine disclosure cannot stand in for Operand's original binding");
    assert!(
        denial.producer_connection_refusal().is_none(),
        "binding affinity refuses before cause lookup"
    );
}

fn assert_program_delivery_is_required(application: &host::OutputHost) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let key = "operand-without-program".to_owned();
    assert!(matches!(
        request
            .advanced()
            .mutate(OperandEdit {
                key: "numeric".to_owned(),
                value: Some(99)
            })
            .without_source()
            .idempotency(&key)
            .execute(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
            ),
        Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)
    ));
}

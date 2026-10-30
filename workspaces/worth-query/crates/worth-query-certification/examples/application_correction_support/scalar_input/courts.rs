//! Real scalar storage, source-created revisions, producer reads and query custody.
use super::super::{
    authentication::*,
    causal_inspection::{independent::retained_query, journey, output_query},
    host,
    output_program::OutputProgram,
};
use super::{
    authoring::{Disclosure, Edit},
    program::Root,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
use worth_query_host::facade::{
    application_entry::*, causal_inspection::*, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

pub fn scalar_storage_connects_only_the_actual_performed_occurrence() {
    let application = host::install_with_outputs();
    let native = application
        .installed_schema()
        .native_contracts()
        .aspect("Number", "ScalarFacts")
        .unwrap();
    assert!(matches!(
        native.contract().shape(),
        worth_foundational::facade::AspectShape::Scalar(
            worth_foundational::facade::ScalarAspectType::UInt64
        )
    ));
    assert!(
        matches!(native.binding(), worth_foundational::facade::AspectBinding::EntityField { field } if field.as_str() == "Value")
    );
    assert_ne!(
        native.contract().key().as_str(),
        "Value",
        "the physical field is not guessed from the aspect name"
    );
    let meanings = application.installed_schema().value_meaning().unwrap();
    let meaning = meanings
        .iter()
        .find(|meaning| meaning.entity() == "Number" && meaning.aspect() == "ScalarFacts")
        .unwrap();
    assert_eq!(meaning.scalar_field(), Some("Value"));
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let data = request
        .query(super::queries::data::Read {
            key: "numeric".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(data.rows().len(), 1);
    assert_eq!((data.rows()[0].input, data.rows()[0].scalar), (5, None));
    drop(data);
    let Err(WorthQueryApplicationRequestQueryDenial::Admission(denial)) = request
        .query(super::queries::inspection::Read {
            key: "numeric".into(),
        })
        .execute()
    else {
        panic!("ordinary data admission cannot grant scalar inspection");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
    );
    assert_eq!(denial.subject(), "scalar-input-inspection");
    let mut first = None;
    for (ordinal, value, expected) in [(0, 3, 14), (1, 7, 18), (2, 3, 14)] {
        let contacts = if ordinal == 0 { None } else { Some(1) };
        let (source, producer, read) =
            perform(&application, value, &format!("scalar-{ordinal}"), contacts);
        let producer = producer.expect("a changed scalar executes its installed producer");
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let (after, row) = retained_query::read(&application, &principal, &scope, &read);
        assert_eq!(row.calculated, expected);
        drop(read);
        let request = application.runtime().request(&principal, &scope);
        let governed = request
            .query(super::queries::inspection::Read {
                key: "numeric".into(),
            })
            .execute_governed(
                super::super::causal_inspection::capability::Inspect::reference(),
                super::super::causal_inspection::capability::InspectCausalEvidence::reference(),
                super::super::causal_inspection::capability::Input::at("numeric"),
            )
            .unwrap();
        assert_eq!(governed.rows().len(), 1);
        assert_eq!(
            (governed.rows()[0].input, governed.rows()[0].scalar),
            (5, Some(value))
        );
        drop(governed);
        let report = request
            .inspect::<output_query::Binding>(&after)
            .why_observed()
            .caused_by::<Disclosure>(&source)
            .reference_only()
            .execute_governed(
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
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
            "scalar-calculated-number"
        );
        assert_eq!(
            readers.observe().active(),
            baseline,
            "report retains no Native read pin"
        );
        report.close();
        if ordinal == 0 {
            first = Some(source);
        } else if ordinal == 2 {
            let old = first.take().unwrap();
            assert_ne!(old.commit_reference(), source.commit_reference());
            refuse_unconnected(&application, &after, &old);
            let (noop, _, reused) = perform(&application, value, "scalar-noop", Some(0));
            assert_ne!(noop.commit_reference(), source.commit_reference());
            drop(reused);
            refuse_unconnected(&application, &after, &noop);
            let foreign = host::install_with_outputs();
            let (foreign_source, _, foreign_read) =
                perform(&foreign, value, "foreign-scalar", None);
            drop(foreign_read);
            let denied = application
                .runtime()
                .request(&principal, &scope)
                .inspect::<output_query::Binding>(&after)
                .why_observed()
                .caused_by::<Disclosure>(&foreign_source)
                .reference_only()
                .execute_governed(
                    journey::access(&application, &principal, &scope),
                    journey::access(&application, &principal, &scope),
                    journey::access(&application, &principal, &scope),
                    journey::access(&application, &principal, &scope),
                )
                .err()
                .expect("genuine foreign issuer is not local source authority");
            assert_eq!(
                denied.kind(),
                WorthQueryApplicationInspectionStopKind::Admission
            );
            assert!(denied.producer_connection_refusal().is_none());
            assert!(denied.resource_refusal().is_none());
        }
        after.close();
        assert_eq!(readers.observe().active(), baseline);
    }
}
fn refuse_unconnected(
    application: &host::OutputHost,
    after: &WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let denied = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<output_query::Binding>(after)
        .why_observed()
        .caused_by::<Disclosure>(source)
        .reference_only()
        .execute_governed(
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
        )
        .err()
        .expect("equal scalar bytes cannot replace the source-created revision");
    assert_eq!(
        denied.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
    );
}
pub(super) fn perform(
    application: &host::OutputHost,
    value: u64,
    key: &str,
    contacts: Option<usize>,
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
        .mutate(Edit {
            key: "numeric".to_owned(),
            value,
        })
        .without_source()
        .idempotency(&key)
        .execute_performed::<OutputProgram, Root>(
            application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let source = match source {
        WorthQueryApplicationPerformedMutationOutcome::Performed(source) => source,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("ordinary scalar source must perform; mutation outcome: {outcome:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
            panic!(
                "ordinary scalar source must perform; retention: {:?}",
                failure.denial()
            )
        }
        WorthQueryApplicationPerformedMutationOutcome::Blocked(blocked) => {
            panic!(
                "ordinary scalar source must perform; blocked: {:?}",
                blocked.outcome()
            )
        }
        WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(source) => {
            panic!(
                "ordinary scalar source must perform; unpublished: {:?}",
                source.initial_cause()
            )
        }
        WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { .. } => {
            panic!("ordinary scalar source must perform; managed cleanup required")
        }
    };
    let receipt = source.receipt().clone();
    let mut started = source
        .start_required_outputs(
            application,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|failure| {
            panic!(
                "scalar source {key} required-output start: {:?}",
                failure.denial()
            )
        });
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(application, &request)
        .unwrap()
    else {
        panic!("actual scalar producer settles");
    };
    // Initial output absence can require reobservation before current settlement.
    // Its causal assertions consume the actual settled receipt and observation.
    if let Some(contacts) = contacts {
        assert_eq!(settled.root_producer_contacts_in_this_demand(), contacts);
    }
    (
        receipt,
        settled.root_receipt().cloned(),
        settled.root_observation().retain(),
    )
}

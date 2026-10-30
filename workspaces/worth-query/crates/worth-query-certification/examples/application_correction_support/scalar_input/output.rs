//! Scalar-only observation cannot borrow the producer's changed Struct sibling.
use super::super::{
    authentication::*,
    causal_inspection::{capability, independent::retained_query, journey},
    host,
};
use super::{authoring::Disclosure, courts::perform};
use worth_query_host::facade::{application_entry::*, causal_inspection::*, primary_graph::*};
pub mod query;
mod retained;

pub fn scalar_output_uses_its_own_performed_revision() {
    let application = host::install_with_outputs();
    let native = application
        .installed_schema()
        .native_contracts()
        .aspect("Number", "ScalarResult")
        .unwrap();
    assert!(matches!(
        native.contract().shape(),
        worth_foundational::facade::AspectShape::Scalar(
            worth_foundational::facade::ScalarAspectType::UInt64
        )
    ));
    assert!(matches!(native.binding(),
        worth_foundational::facade::AspectBinding::EntityField { field }
            if field.as_str() == "Total"));
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let Err(WorthQueryApplicationRequestQueryDenial::Admission(denied)) = request
        .query(query::Read {
            key: "numeric".into(),
        })
        .execute()
    else {
        panic!("ordinary query access cannot grant scalar-output disclosure")
    };
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
    );
    assert_eq!(denied.subject(), "scalar-output");
    let initial = request
        .query(query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input::at("numeric"),
        )
        .unwrap();
    assert_eq!(initial.rows().len(), 1);
    assert_eq!(initial.rows()[0].total, None);
    drop(initial);
    let mut first = None;
    for (ordinal, value, expected) in [(0, 3, 38), (1, 2, 31), (2, 3, 38)] {
        let contacts = if ordinal == 0 { None } else { Some(1) };
        let (source, producer, read) = perform(
            &application,
            value,
            &format!("scalar-output-{ordinal}"),
            contacts,
        );
        let producer = producer.expect("changed input runs the actual producer");
        let (after, total) = retained::read(&application, &principal, &scope, &read);
        drop(read);
        assert_eq!(total, Some(expected));
        let report = request
            .inspect::<query::Binding>(&after)
            .why_observed()
            .caused_by::<Disclosure>(&source)
            .reference_only()
            .execute_governed(
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
            )
            .unwrap_or_else(|denial| {
                panic!(
                    "scalar output {ordinal} inspection: kind {:?}, connection {:?}, output {:?}, resource {:?}, canonical {:?}",
                    denial.kind(),
                    denial.producer_connection_refusal(),
                    denial.producer_output_refusal(),
                    denial.resource_refusal(),
                    denial.canonical_refusal(),
                )
            });
        assert_eq!(
            report.producer_connection().source_commit(),
            source.commit_reference()
        );
        assert_eq!(
            report.producer_connection().producer_commit(),
            producer.commit_reference()
        );
        assert_ne!(
            report.producer_connection().output_publication_commit(),
            source.commit_reference(),
            "the input edit did not author the producer's Scalar output",
        );
        assert_eq!(
            report.producer_connection().output_role(),
            "scalar-calculated-number"
        );
        assert_eq!(readers.observe().active(), baseline);
        report.close();
        if ordinal == 0 {
            first = Some(source);
        } else if ordinal == 2 {
            let old = first.take().unwrap();
            assert_ne!(old.commit_reference(), source.commit_reference());
            refuse(
                &application,
                &after,
                &old,
                WorthQueryProducerConnectionDenial::SourceInputUnconnected,
            );
        }
        after.close();
        assert_eq!(readers.observe().active(), baseline);
    }
    // 7 changes the real input and Calculated14->18, but Scalar Total stays38.
    // That changed Struct publication cannot provide this query's output effect.
    let (source, producer, read) = perform(&application, 7, "scalar-output-equal", Some(1));
    assert!(producer.is_some());
    let (structure, row) = retained_query::read(&application, &principal, &scope, &read);
    assert_eq!(row.calculated, 18);
    structure.close();
    let (after, total) = retained::read(&application, &principal, &scope, &read);
    assert_eq!(total, Some(38));
    drop(read);
    refuse(
        &application,
        &after,
        &source,
        WorthQueryProducerConnectionDenial::OutputUnconnected,
    );
    after.close();
    let (noop, _, reused) = perform(&application, 7, "scalar-output-noop", Some(0));
    let (after, total) = retained::read(&application, &principal, &scope, &reused);
    assert_eq!(total, Some(38));
    drop(reused);
    refuse(
        &application,
        &after,
        &noop,
        WorthQueryProducerConnectionDenial::SourceInputUnconnected,
    );
    after.close();
    assert_eq!(readers.observe().active(), baseline);
}

fn refuse(
    application: &host::OutputHost,
    after: &WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
    expected: WorthQueryProducerConnectionDenial,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let denied = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(after)
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
        .expect("scalar connection requires its own original input and output effects");
    assert_eq!(denied.producer_connection_refusal(), Some(expected));
    if expected == WorthQueryProducerConnectionDenial::OutputUnconnected {
        assert_eq!(
            denied.producer_output_refusal(),
            Some(WorthQueryProducerOutputConnectionDenial::NativeRevisionNotCreatedByProducer),
            "equal Scalar output retains its older revision despite the changed Struct sibling",
        );
    } else {
        assert!(denied.producer_output_refusal().is_none());
    }
}

//! Shared Native commit coordinates do not identify an authored source as its producer.
use super::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::{
    admitted_capture::checkpoint,
    application_output_demand::settlement::observation::OriginalSettledProducerSource,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation;
use worth_relational::facade::publication::PublishedAuthoritativeFieldEffect as Effect;

pub(super) fn paired_changes(
    application: &host::OutputHost,
    source: &WorthQueryApplicationCommitReceipt,
    read: &WorthQueryApplicationReadObservation,
) {
    let retained = application
        .with_host_advancement(|phase| {
            read.select_on(&phase, application.runtime())
                .unwrap()
                .retain_application_read()
        })
        .unwrap();
    let supplier = retained.original_settled_producer_for_test().unwrap();
    let OriginalSettledProducerSource::Committed {
        changes,
        original_publication,
        ..
    } = supplier.source()
    else {
        panic!("this changed scalar retains an actual committed producer")
    };
    let producer = original_publication
        .as_ref()
        .map_or(changes, |old| old.changes_for_test());
    if source.aftermath_causality().is_some() {
        assert_ne!(producer.commit_reference(), source.commit_reference());
        return;
    }
    assert_eq!(
        producer.commit_reference(),
        source.commit_reference(),
        "the original producer co-committed with its source"
    );
    let (_, _, entity_name, entity) = supplier
        .correspondence()
        .native_witness_roles_for_test()
        .next()
        .unwrap();
    let layout = application
        .runtime()
        .runtime_for_test()
        .primary_provider_for_test()
        .graph_for_test()
        .layout_for_test();
    let native = application
        .installed_schema()
        .native_contracts()
        .aspect("Number", super::super::ScalarResult::reference().name())
        .unwrap();
    application
        .with_host_advancement(|phase| {
            let execution = phase
                .checked_execution_request_for(application.runtime().product_runtime())
                .unwrap();
            let mut effects = None;
            execution
                .run_native_with_checkpoint(|context| {
                    let coordinate = layout
                        .resolve_declared_field_coordinate_for_test(
                            application.installed_schema().native_contracts(),
                            entity_name,
                            native.contract().key(),
                            &worth_foundational::facade::FieldKey::new("Total").unwrap(),
                            |work| checkpoint(context, work),
                        )
                        .unwrap()
                        .unwrap();
                    effects = Some((
                        source
                            .committed_changes()
                            .source_field_effect_for_test(context, entity, coordinate)
                            .unwrap(),
                        producer
                            .field_effect_for_test(context, entity, coordinate)
                            .unwrap(),
                    ));
                })
                .unwrap();
            assert_eq!(
                effects,
                Some((Effect::Unchanged, Effect::Present)),
                "actual source prefix excludes Total; actual producer commit includes it"
            );
        })
        .unwrap();
    refuse_source_as_producer(application, source, &retained);
}

fn refuse_source_as_producer(
    application: &host::OutputHost,
    source: &WorthQueryApplicationCommitReceipt,
    retained: &worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
) {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::settlement::observation::OriginalSettledProducerObservation;
    let supplier = retained.original_settled_producer_for_test().unwrap();
    let mut false_source = supplier.source().clone();
    let OriginalSettledProducerSource::Committed {
        changes,
        settlement,
        publication,
        original_publication,
        ..
    } = &mut false_source
    else {
        unreachable!()
    };
    *changes = source.committed_changes().clone();
    *settlement = None;
    *publication = None;
    *original_publication = None;
    // A deliberately false supplier claim uses the REAL source receipt, with
    // its actual role roster, in the real retained query and current admission.
    let false_supplier = OriginalSettledProducerObservation::new_for_test(
        *supplier.runtime_authority_for_test(),
        supplier.schema_for_test().clone(),
        supplier.read().clone(),
        false_source,
        source.retain_output_correspondence_for_test(),
        None,
    );
    let false_read = retained.with_original_producer_for_test(std::sync::Arc::new(false_supplier));
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (after, total) = super::retained::read_core(application, &principal, &scope, &false_read);
    assert!(total.is_some());
    let denial = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<super::query::Binding>(&after)
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
        .expect(
            "a real source receipt cannot serve as its producer, despite the shared Native commit",
        );
    assert_eq!(
        denial.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::OutputUnconnected)
    );
    assert_eq!(
        denial.producer_output_refusal(),
        Some(WorthQueryProducerOutputConnectionDenial::NoDisclosedOutputCandidate)
    );
    after.close();
}

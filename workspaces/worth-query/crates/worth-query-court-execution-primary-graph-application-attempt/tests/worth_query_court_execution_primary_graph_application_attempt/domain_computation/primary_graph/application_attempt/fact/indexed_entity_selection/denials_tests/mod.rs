use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_query_execution::facade::application_inspection::IndexedSelectionReobserveDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::observe_checked;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountNote;

#[test]
fn indexed_reobservation_preserves_native_request_and_lookup_denials() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let field = AccountNote::reference();
    let locator = (*graph.layout_for_test())
        .field_locator_for_test(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let kind = (*graph.layout_for_test())
        .entity_kind_for_test(AccountIdentity::reference().entity())
        .unwrap();
    let value = StringApplicationValueBinding::encode(&"reviewed".to_owned()).unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            graph
                .integration_handle_for_test()
                .with_runtime(request_execution, |runtime| {
                    for (limit, expected) in [
                        (0, BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit),
                        (2, BoundedEntityFieldLookupDenialKind::IndexNotInstalled),
                    ] {
                        let result = observe_checked(
                            runtime,
                            &host_custody.borrowed_request_for_test(),
                            selected.application_basis_for_test().snapshot_handle(),
                            DerivedIndexId(u64::MAX),
                            kind,
                            locator.clone(),
                            value.clone(),
                            limit,
                        );
                        assert_eq!(
                            result.unwrap_err(),
                            IndexedSelectionReobserveDenial::Lookup(expected),
                            "the native denial must survive Query's fact observation boundary"
                        );
                    }
                })
        })
        .expect("the fixture read owns its installed host call");
}

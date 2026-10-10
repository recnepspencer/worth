use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};

use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountIdentity, AccountNote,
};

#[test]
fn indexed_reobservation_preserves_native_request_and_lookup_denials() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = AccountNote::reference();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let kind = graph
        .layout
        .entity_kind(AccountIdentity::reference().entity())
        .unwrap();
    let value = StringApplicationValueBinding::encode(&"reviewed".to_owned()).unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            graph
                .integration_handle()
                .with_runtime(request_execution, |runtime| {
                    for (limit, expected) in [
                        (0, BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit),
                        (2, BoundedEntityFieldLookupDenialKind::IndexNotInstalled),
                    ] {
                        let result = observe_checked(
                            runtime,
                            &host_custody.borrowed_request_for_test(),
                            selected.application_basis().snapshot_handle(),
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

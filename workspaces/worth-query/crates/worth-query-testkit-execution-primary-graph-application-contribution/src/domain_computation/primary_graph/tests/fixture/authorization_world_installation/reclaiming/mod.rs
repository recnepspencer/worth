//! Explicit reclaiming Native policy is supplied before actual schema installation.
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::*;
use worth_relational::facade::config::RelationalRuntimeProfile;
use worth_relational::facade::runtime::RelationalRuntimeApi;

pub fn install_reclaiming_authorization_world(
    spec: AuthorizationWorldSpec<'_>,
) -> AuthorizationWorld {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|phase| {
        let native = RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::AiWorkflow)
            .build()
            .expect("the reclaiming authorization fixture admits its Native runtime");
        let mut prepared = prepare_authorization_world(
            &phase, spec.resources, Some(native),
            worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
        );
        populate_authorization_world(&mut prepared, &spec);
        publish_authorization_world(&phase, prepared, spec.admission_clock)
    })
}

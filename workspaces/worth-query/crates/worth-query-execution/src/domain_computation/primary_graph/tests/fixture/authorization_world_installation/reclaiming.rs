//! Explicit reclaiming Native policy is supplied before actual schema installation.
use super::*;
use worth_relational::facade::{config::RelationalRuntimeProfile, runtime::RelationalRuntimeApi};

pub(in crate::domain_computation::primary_graph::tests::fixture) fn install_reclaiming_authorization_world(
    spec: AuthorizationWorldSpec<'_>,
) -> AuthorizationWorld {
    crate::domain_computation::primary_graph::with_test_advancement(|phase| {
        let native = RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::AiWorkflow)
            .build()
            .expect("the reclaiming authorization fixture admits its Native runtime");
        let mut prepared = prepare_authorization_world(
            &phase, spec.resources, Some(native),
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
        );
        populate_authorization_world(&mut prepared, &spec);
        publish_authorization_world(&phase, prepared)
    })
}

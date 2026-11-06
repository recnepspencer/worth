use crate::domain_computation::primary_graph::tests::fixture::capability::*;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;

pub fn bind_same_resource_unrelated_grants(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    unrelated: usize,
) {
    crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant(
        bootstrap,
    );
    for ordinal in 0..unrelated {
        let grant = format!("same-resource-unrelated-{ordinal}");
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant_entity(bootstrap, &grant, 70, 130, 0, 100);
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_actor_relation(
            bootstrap,
            CapabilityGrantee::reference(),
            &format!("{grant}-grantee"),
            "principal-1",
            &grant,
        );
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_actor_relation(
            bootstrap,
            CapabilityGrantor::reference(),
            &format!("{grant}-grantor"),
            "principal-1",
            &grant,
        );
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_resource(
            bootstrap,
            &grant,
            "account-1",
        );
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_related(
            bootstrap,
            &grant,
            "account-2",
        );
    }
}

pub fn bind_exact_pair_grants(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    count: usize,
) {
    for ordinal in 0..count {
        let grant = format!("exact-pair-{ordinal}");
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant_entity(bootstrap, &grant, 90, 110, 0, 50);
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_actor_relation(
            bootstrap,
            CapabilityGrantee::reference(),
            &format!("{grant}-grantee"),
            "principal-0",
            &grant,
        );
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_actor_relation(
            bootstrap,
            CapabilityGrantor::reference(),
            &format!("{grant}-grantor"),
            "principal-0",
            &grant,
        );
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_resource(
            bootstrap,
            &grant,
            "account-1",
        );
        crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_related(
            bootstrap,
            &grant,
            "account-2",
        );
    }
}

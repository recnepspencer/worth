use crate::domain_computation::primary_graph::tests::fixture::capability::*;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;

pub use crate::domain_computation::primary_graph::tests::fixture::capability_seed::composition::bind_composed_grant;
pub use crate::domain_computation::primary_graph::tests::fixture::capability_seed::composition::CapabilityCompositionScenario;

pub fn bind_grant(bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>) {
    bind_grant_window(bootstrap, "capability-1", 90, 110, 50);
}

pub fn bind_command_grant(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    key: &str,
    principal: &str,
    resource: &str,
    action: CapabilityAction,
) {
    bind_grant_entity_with_action(bootstrap, key, 90, 110, 0, 0, action);
    bind_actor_relation(
        bootstrap,
        CapabilityGrantee::reference(),
        &format!("{key}-grantee"),
        principal,
        key,
    );
    bind_actor_relation(
        bootstrap,
        CapabilityGrantor::reference(),
        &format!("{key}-grantor"),
        principal,
        key,
    );
    bind_resource(bootstrap, key, resource);
}

pub fn bind_future_replacement_grant(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
) {
    bind_grant_window(bootstrap, "capability-2", 111, 200, 50);
}

pub fn bind_delegated_grants(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    links: usize,
    unrelated: usize,
) {
    assert!(
        links <= 2,
        "the installed fixture contract admits at most two links"
    );
    if links == 2 {
        bind_delegation_root(bootstrap, "capability-grandparent", 80, 120, 3, 100);
    }
    if links >= 1 {
        bind_grant_entity(bootstrap, "capability-parent", 90, 110, 2, 75);
        bind_actor_relation(
            bootstrap,
            CapabilityGrantee::reference(),
            "capability-parent-grantee",
            "principal-1",
            "capability-parent",
        );
        bind_actor_relation(
            bootstrap,
            CapabilityGrantor::reference(),
            "capability-parent-grantor",
            "principal-1",
            "capability-parent",
        );
        bind_resource(bootstrap, "capability-parent", "account-1");
        bind_related(bootstrap, "capability-parent", "account-2");
        if links == 2 {
            bind_parent(bootstrap, "capability-parent", "capability-grandparent");
        }
    }

    bind_delegation_root(bootstrap, "capability-alternate", 70, 130, 4, 100);

    bind_grant_entity(bootstrap, "capability-child", 95, 105, 1, 50);
    bind_actor_relation(
        bootstrap,
        CapabilityGrantee::reference(),
        "capability-child-grantee",
        "principal-0",
        "capability-child",
    );
    bind_actor_relation(
        bootstrap,
        CapabilityGrantor::reference(),
        "capability-child-grantor",
        "principal-1",
        "capability-child",
    );
    bind_actor_relation(
        bootstrap,
        CapabilityCustodian::reference(),
        "capability-child-custodian",
        "principal-0",
        "capability-child",
    );
    bind_resource(bootstrap, "capability-child", "account-1");
    bind_related(bootstrap, "capability-child", "account-2");
    if links >= 1 {
        bind_parent(bootstrap, "capability-child", "capability-parent");
    }
    for ordinal in 0..unrelated {
        bind_unrelated_root(bootstrap, ordinal);
    }
}

pub fn bind_unrelated_root(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    ordinal: usize,
) {
    let grant = format!("unrelated-capability-{ordinal}");
    bind_grant_entity(bootstrap, &grant, 70, 130, 4, 100);
    bind_actor_relation(
        bootstrap,
        CapabilityGrantee::reference(),
        &format!("{grant}-grantee"),
        "principal-0",
        &grant,
    );
    bind_actor_relation(
        bootstrap,
        CapabilityGrantor::reference(),
        &format!("{grant}-grantor"),
        "principal-1",
        &grant,
    );
    bind_actor_relation(
        bootstrap,
        CapabilityCustodian::reference(),
        &format!("{grant}-custodian"),
        "principal-1",
        &grant,
    );
    bind_resource(bootstrap, &grant, "account-2");
    bind_related(bootstrap, &grant, "account-1");
}

pub fn bind_delegation_root(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    grant: &str,
    not_before: u64,
    not_after: u64,
    remaining: u64,
    amount: u64,
) {
    bind_grant_entity(bootstrap, grant, not_before, not_after, remaining, amount);
    bind_actor_relation(
        bootstrap,
        CapabilityGrantee::reference(),
        &format!("{grant}-grantee"),
        "principal-1",
        grant,
    );
    bind_actor_relation(
        bootstrap,
        CapabilityGrantor::reference(),
        &format!("{grant}-grantor"),
        "principal-1",
        grant,
    );
    bind_resource(bootstrap, grant, "account-1");
    bind_related(bootstrap, grant, "account-2");
}

pub fn bind_grant_window(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    key: &str,
    not_before: u64,
    not_after: u64,
    amount: u64,
) {
    bind_grant_entity(bootstrap, key, not_before, not_after, 0, amount);
    bind_actor_relation(
        bootstrap,
        CapabilityGrantee::reference(),
        &format!("{key}-grantee"),
        "principal-0",
        key,
    );
    bind_actor_relation(
        bootstrap,
        CapabilityGrantor::reference(),
        &format!("{key}-grantor"),
        "principal-0",
        key,
    );
    bind_resource(bootstrap, key, "account-1");
    bind_related(bootstrap, key, "account-2");
}

pub fn bind_grant_entity(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    key: &str,
    not_before: u64,
    not_after: u64,
    delegation_limit: u64,
    amount: u64,
) {
    bind_grant_entity_with_action(
        bootstrap,
        key,
        not_before,
        not_after,
        delegation_limit,
        amount,
        CapabilityAction::Touch,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn bind_grant_entity_with_action(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    key: &str,
    not_before: u64,
    not_after: u64,
    delegation_limit: u64,
    amount: u64,
    action: CapabilityAction,
) {
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                CapabilityGrant::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(CapabilityIdentity::reference(), key.to_owned())
            .field(CapabilityActionField::reference(), action)
            .field(
                CapabilityPurposeField::reference(),
                CapabilityPurpose::AccountMaintenance,
            )
            .field(
                CapabilityDisclosureField::reference(),
                CapabilityDisclosure::AccountActivity,
            )
            .field(CapabilityStatusField::reference(), CapabilityStatus::Active)
            .field(CapabilityWorkflowField::reference(), "open".to_owned())
            .field(CapabilityNotBeforeField::reference(), not_before)
            .field(CapabilityNotAfterField::reference(), not_after)
            .field(CapabilityAmountField::reference(), amount)
            .field(
                CapabilityDelegationLimitField::reference(),
                delegation_limit,
            ),
        )
        .unwrap();
}

pub fn bind_related(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    grant: &str,
    related: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            CapabilityRelated::reference(),
            format!("{grant}-related"),
            WorthQueryApplicationEntityKey::new(grant).unwrap(),
            WorthQueryApplicationEntityKey::new(related).unwrap(),
        ))
        .unwrap();
}

pub fn bind_actor_relation<Relation>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Principal,
        CapabilityGrant,
    >,
    key: &str,
    principal: &str,
    grant: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation,
            key,
            WorthQueryApplicationEntityKey::new(principal).unwrap(),
            WorthQueryApplicationEntityKey::new(grant).unwrap(),
        ))
        .unwrap();
}

pub fn bind_resource(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    grant: &str,
    resource: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            CapabilityResource::reference(),
            format!("{grant}-resource"),
            WorthQueryApplicationEntityKey::new(grant).unwrap(),
            WorthQueryApplicationEntityKey::new(resource).unwrap(),
        ))
        .unwrap();
}

pub fn bind_parent(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    child: &str,
    parent: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            CapabilityParent::reference(),
            format!("{child}-parent"),
            WorthQueryApplicationEntityKey::new(child).unwrap(),
            WorthQueryApplicationEntityKey::new(parent).unwrap(),
        ))
        .unwrap();
}

#[path = "composition/mod.rs"]
pub mod composition;

//! Genuine grant population and installed capability declaration for these courts.

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::capability::declarations::*;
use worth_query_declaration::facade::application_schema::*;
pub fn declare<S: OrbitModel>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::capability::contract::declare(schema)
}
pub fn seed<const O: bool>(bootstrap: &mut WorthQueryPrimaryGraphBootstrap<OrbitSchema<O>>) {
    if !O {
        return;
    }
    let grant =
        || WorthQueryApplicationEntityKey::<OrbitSchema<O>, Grant>::new("orbit-grant").unwrap();
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(Grant::reference(), grant())
                .field(GrantAction::at("GrantAction"), false)
                .field(GrantPurpose::at("GrantPurpose"), false)
                .field(GrantActive::at("GrantActive"), true)
                .field(GrantWorkflow::at("GrantWorkflow"), false)
                .field(GrantDisclosure::at("GrantDisclosure"), 0)
                .field(GrantFrom::at("GrantFrom"), 0)
                .field(GrantUntil::at("GrantUntil"), 4_102_444_800_u64)
                .field(GrantLimit::at("GrantLimit"), 1),
        )
        .unwrap();
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            Holder::reference(),
            "orbit-holder",
            WorthQueryApplicationEntityKey::<OrbitSchema<O>, Principal>::new("principal-0")
                .unwrap(),
            grant(),
        ))
        .unwrap();
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            Issuer::reference(),
            "orbit-issuer",
            WorthQueryApplicationEntityKey::<OrbitSchema<O>, Principal>::new("principal-0")
                .unwrap(),
            grant(),
        ))
        .unwrap();
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            Resource::reference(),
            "orbit-resource",
            grant(),
            WorthQueryApplicationEntityKey::<OrbitSchema<O>, Node>::new("node-0").unwrap(),
        ))
        .unwrap();
}

#[path = "contract/mod.rs"]
pub mod contract;
#[path = "declarations/mod.rs"]
pub mod declarations;

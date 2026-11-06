//! Genuine grant population and installed capability declaration for these courts.
mod contract;
pub(crate) mod declarations;
use super::super::schema::*;
use crate::domain_computation::primary_graph::*;
use declarations::*;
use worth_query_declaration::facade::application_schema::*;
pub(super) fn declare<S: OrbitModel>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    contract::declare(schema)
}
pub(in crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits) fn seed<
    const O: bool,
>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<OrbitSchema<O>>,
) {
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

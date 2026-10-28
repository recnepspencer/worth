//! Genuine grant population and installed capability declaration for these courts.
mod contract;
pub(super) mod declarations;
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
                .field(GrantAction::reference(), false)
                .field(GrantPurpose::reference(), false)
                .field(GrantActive::reference(), true)
                .field(GrantWorkflow::reference(), false)
                .field(GrantDisclosure::reference(), 0)
                .field(GrantFrom::reference(), 0)
                .field(GrantUntil::reference(), 4_102_444_800_u64)
                .field(GrantLimit::reference(), 1),
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

//! Shared topology meaning used by the consumer and native owner fixture.
use super::fixture_values::PositiveLength;
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_unit, worth_query_value_binding,
};

/// What the shared topology model asks of the schema that declares it. A
/// program's own schema policy, such as the consumer fixture's
/// `TopologySchemaBinding`, stays with that program.
pub trait TopologyModelBinding: ApplicationSchema {
    /// The work the checked planar invariant may spend.
    fn planar_invariant_work() -> u64 {
        super::planar_topology::PLANAR_CHECKED_WORK
    }
}

worth_query_value_binding! {
    pub TopologyLengthBinding for PositiveLength {
        identity: "worth.query.certification.positive-length.v1",
        scalar: UInt64,
        unit: "Metre",
        frame: "worth.frames.model-local.v1",
        encode: PositiveLength::get,
        decode: PositiveLength::new,
    }
}

worth_query_entity!(pub Body for Schema: TopologyModelBinding);
worth_query_unit!(pub Metre(()) for Schema: TopologyModelBinding);
worth_query_aspect!(
    pub Geometry for Schema: TopologyModelBinding, Body;
    identity = AspectIdentity(0x9174_1001),
    revision = AspectContractRevision(1),
);
worth_query_field!(
    pub Length for Schema: TopologyModelBinding, Body, Geometry:
    PositiveLength => TopologyLengthBinding, unit Metre, read_write, equality
);

pub(super) fn declare<Schema: TopologyModelBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use super::*;
    schema
        .entity(Body::reference::<Schema>())
        .unit(Metre::reference::<Schema>())
        .aspect(Body::reference::<Schema>(), Geometry::reference::<Schema>())
        .field(Body::reference::<Schema>(), Length::reference::<Schema>())
        .aspect(
            Body::reference::<Schema>(),
            PlanarPosition::reference::<Schema>(),
        )
        .field(Body::reference::<Schema>(), BodyKey::reference::<Schema>())
        .field(
            Body::reference::<Schema>(),
            PositionX::reference::<Schema>(),
        )
        .field(
            Body::reference::<Schema>(),
            PositionY::reference::<Schema>(),
        )
        .relation(
            PlanarSuccessor::reference::<Schema>(),
            Body::reference::<Schema>(),
            Body::reference::<Schema>(),
        )
        .relation(
            PlanarDiscoverySource::reference::<Schema>(),
            Body::reference::<Schema>(),
            Body::reference::<Schema>(),
        )
        .relation(
            UndeclaredPlanarRelation::reference::<Schema>(),
            Body::reference::<Schema>(),
            Body::reference::<Schema>(),
        )
        .invariant(planar_turn_invariant::<Schema>())
        .entity(ExternalPrincipalMapping::reference::<Schema>())
        .entity(Principal::reference::<Schema>())
        .aspect(
            ExternalPrincipalMapping::reference::<Schema>(),
            ExternalIdentity::reference::<Schema>(),
        )
        .aspect(
            Principal::reference::<Schema>(),
            PrincipalFacts::reference::<Schema>(),
        )
        .field(
            ExternalPrincipalMapping::reference::<Schema>(),
            ExternalIdentityField::reference::<Schema>(),
        )
        .field(
            ExternalPrincipalMapping::reference::<Schema>(),
            MappingStatusField::reference::<Schema>(),
        )
        .field(
            Principal::reference::<Schema>(),
            PrincipalIdentity::reference::<Schema>(),
        )
        .relation(
            MappingTarget::reference::<Schema>(),
            ExternalPrincipalMapping::reference::<Schema>(),
            Principal::reference::<Schema>(),
        )
        .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
}

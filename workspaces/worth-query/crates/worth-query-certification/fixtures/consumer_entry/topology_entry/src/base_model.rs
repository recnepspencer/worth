//! Shared topology meaning used by the consumer and native owner fixture.
use super::fixture_values::PositiveLength;
use super::runtime_api::application_contribution::{
    WorthQueryDecisionContextDependencies, WorthQueryProducerInputReuseContract,
};
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_unit, worth_query_value_binding,
};

pub trait TopologySchemaBinding: ApplicationSchema {
    /// Isolated consumer variants can require the real planar producer binding
    /// to belong to their application program; existing schemas keep their policy.
    const PLANAR_REQUIRES_APPLICATION_PROGRAM: bool = false;
    const PLANAR_EDIT_PROJECTION_WORK: Option<usize> = Some(8_192);
    fn planar_invariant_work() -> Option<std::num::NonZeroU64> {
        std::num::NonZeroU64::new(super::planar_topology::PLANAR_CHECKED_WORK)
    }
    const ROOT_INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
    /// The final producer's decision-reuse declaration is part of each installed
    /// fixture schema, so opt-out journeys use the ordinary production installer.
    const FINAL_INPUT_REUSE: Option<WorthQueryProducerInputReuseContract> =
        Some(WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::NONE,
        ));
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

worth_query_entity!(pub Body for Schema: TopologySchemaBinding);
worth_query_unit!(pub Metre(()) for Schema: TopologySchemaBinding);
worth_query_aspect!(
    pub Geometry for Schema: TopologySchemaBinding, Body;
    identity = AspectIdentity(0x9174_1001),
    revision = AspectContractRevision(1),
);
worth_query_field!(
    pub Length for Schema: TopologySchemaBinding, Body, Geometry:
    PositiveLength => TopologyLengthBinding, unit Metre, read_write, equality
);

pub(super) fn declare<Schema: TopologySchemaBinding>(
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

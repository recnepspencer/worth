#![forbid(unsafe_code)]

use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::{
    application_schema::ApplicationSchema, worth_query_application_contribution,
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_unit,
    worth_query_value_binding,
};

mod alternate_output;
mod application_program;
mod contribution;
mod final_output;
mod handler;
pub use alternate_output::*;
pub use application_program::*;
pub use contribution::TopologyConfiguration;
pub use final_output::*;
mod mutation;
mod mutation_identity;
mod planar_invariant;
mod planar_output_read;
mod planar_read;
mod planar_topology;
mod principal;
mod prior_cycle_adjustment;
mod producer;
mod readiness;
mod source_adjustment;
mod vertex_replacement;
pub use vertex_replacement::*;

pub use handler::*;
pub use mutation::*;
pub use planar_invariant::*;
pub use planar_output_read::*;
pub use planar_read::*;
pub use planar_topology::*;
pub use principal::*;
pub use prior_cycle_adjustment::*;
pub use producer::*;
pub use readiness::InitialPlanarReadiness;
pub use source_adjustment::*;

pub trait TopologySchemaBinding: ApplicationSchema {}

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

worth_query_application_contribution! {
    pub contribution TopologyContribution for Schema: TopologySchemaBinding {
        identity: "worth.query.certification.topology.v1",
        members: |schema| {
            let schema = alternate_output::declare_alternate_output(schema);
            let schema = vertex_replacement::declare_vertex_replacement(schema);
            let schema = prior_cycle_adjustment::declare_prior_cycle_adjustment(schema);
            let schema = source_adjustment::declare_planar_source_adjustment(schema);
            let schema = final_output::declare_final_output(schema);
            schema
                .entity(Body::reference::<Schema>())
                .unit(Metre::reference::<Schema>())
                .aspect(Body::reference::<Schema>(), Geometry::reference::<Schema>())
                .field(Body::reference::<Schema>(), Length::reference::<Schema>())
                .aspect(Body::reference::<Schema>(), PlanarPosition::reference::<Schema>())
                .field(Body::reference::<Schema>(), BodyKey::reference::<Schema>())
                .field(Body::reference::<Schema>(), PositionX::reference::<Schema>())
                .field(Body::reference::<Schema>(), PositionY::reference::<Schema>())
                .relation(
                    PlanarSuccessor::reference::<Schema>(),
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
                .aspect(ExternalPrincipalMapping::reference::<Schema>(), ExternalIdentity::reference::<Schema>())
                .aspect(Principal::reference::<Schema>(), PrincipalFacts::reference::<Schema>())
                .field(ExternalPrincipalMapping::reference::<Schema>(), ExternalIdentityField::reference::<Schema>())
                .field(ExternalPrincipalMapping::reference::<Schema>(), MappingStatusField::reference::<Schema>())
                .field(Principal::reference::<Schema>(), PrincipalIdentity::reference::<Schema>())
                .relation(MappingTarget::reference::<Schema>(), ExternalPrincipalMapping::reference::<Schema>(), Principal::reference::<Schema>())
                .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
                .operation(MutatePlanar::reference::<Schema>().definition().no_external_effect().no_aftermath().finish())
                .operation_decision_fact_budget(MutatePlanar::reference::<Schema>(), 64)
                .operation_projection_work_budget(MutatePlanar::reference::<Schema>(), 256)
                .operation_read_entity(MutatePlanar::reference::<Schema>(), Body::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), BodyKey::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), PositionX::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), PositionY::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), Length::reference::<Schema>())
                .operation_read_relation(MutatePlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .operation_create(MutatePlanar::reference::<Schema>(), Body::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), BodyKey::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), PositionX::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), PositionY::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), Length::reference::<Schema>())
                .operation_link(MutatePlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .operation_unlink(MutatePlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .application_mutation_binding::<PlanarMutationBinding<Schema>>()
                .application_query(planar_query_definition::<Schema>())
                .application_query_binding::<PlanarReadBinding<Schema>>()
                .application_query(planar_output_query_definition::<Schema>())
                .application_query_binding::<PlanarOutputReadBinding<Schema>>()
        }
    }
}

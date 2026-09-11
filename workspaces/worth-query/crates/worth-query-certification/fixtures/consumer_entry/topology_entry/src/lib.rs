#![forbid(unsafe_code)]

use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::{
    application_schema::ApplicationSchema, worth_query_application_contribution,
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_unit,
    worth_query_value_binding,
};

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
            schema
                .entity(Body::reference::<Schema>())
                .unit(Metre::reference::<Schema>())
                .aspect(Body::reference::<Schema>(), Geometry::reference::<Schema>())
                .field(Body::reference::<Schema>(), Length::reference::<Schema>())
        }
    }
}

//! Three output roles and two actual internal relations across codec cutover.
use super::{binding, model, output as wire_model};
#[path = "related/cause.rs"]
mod cause;
#[path = "related/continuation.rs"]
mod continuation;
#[path = "related/derived_query.rs"]
mod derived_query;
#[path = "edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
mod edit;
#[path = "related/host.rs"]
mod host;
#[path = "related/journey.rs"]
mod journey;
#[path = "related/limits.rs"]
mod limits;
#[path = "related/live.rs"]
mod live;
#[path = "related/live_query.rs"]
mod live_query;
#[path = "related/membership.rs"]
mod membership;
#[path = "related/output.rs"]
mod output;
#[path = "related/page_query.rs"]
mod page_query;
#[path = "related/paired.rs"]
mod paired;
#[path = "related/paired_native.rs"]
mod paired_native;
#[path = "related/paired_query.rs"]
mod paired_query;
#[path = "related/producer.rs"]
mod producer;
#[path = "related/program.rs"]
mod program;
#[path = "related/projection.rs"]
mod projection;
#[path = "related/publish.rs"]
mod publish;
#[path = "../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
mod query;
#[path = "readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
mod readiness;
#[path = "related/retained.rs"]
mod retained;
#[path = "related/retire.rs"]
mod retire;
#[path = "related/retirement.rs"]
mod retirement;
#[path = "related/schema.rs"]
mod schema;
#[path = "related/settlement.rs"]
mod settlement;
#[path = "related/successor.rs"]
mod successor;

fn heavy_value() -> model::NestedValue {
    journey::heavy()
}

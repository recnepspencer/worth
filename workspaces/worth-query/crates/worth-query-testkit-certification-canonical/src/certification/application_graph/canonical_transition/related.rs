//! Three output roles and two actual internal relations across codec cutover.
pub use super::{binding, model, output as wire_model};
#[path = "related/cause.rs"]
pub mod cause;
#[path = "related/continuation.rs"]
pub mod continuation;
#[path = "related/derived_query.rs"]
pub mod derived_query;
#[path = "edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod edit;
#[path = "related/host.rs"]
pub mod host;
#[path = "related/journey.rs"]
pub mod journey;
#[path = "related/limits.rs"]
pub mod limits;
#[path = "related/live.rs"]
pub mod live;
#[path = "related/live_query.rs"]
pub mod live_query;
#[path = "related/membership.rs"]
pub mod membership;
#[path = "related/output.rs"]
pub mod output;
#[path = "related/page_query.rs"]
pub mod page_query;
#[path = "related/paired.rs"]
pub mod paired;
#[path = "related/paired_native.rs"]
pub mod paired_native;
#[path = "related/paired_query.rs"]
pub mod paired_query;
#[path = "related/producer.rs"]
pub mod producer;
#[path = "related/program.rs"]
pub mod program;
#[path = "related/projection.rs"]
pub mod projection;
#[path = "related/publish.rs"]
pub mod publish;
#[path = "../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod query;
#[path = "readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod readiness;
#[path = "related/retained.rs"]
pub mod retained;
#[path = "related/retire.rs"]
pub mod retire;
#[path = "related/retirement.rs"]
pub mod retirement;
#[path = "related/schema.rs"]
pub mod schema;
#[path = "related/settlement.rs"]
pub mod settlement;
#[path = "related/successor.rs"]
pub mod successor;

pub fn heavy_value() -> model::NestedValue {
    journey::heavy()
}

#[path = "related/companion.rs"]
pub mod companion;

#[path = "related/companion_transition.rs"]
pub mod companion_transition;
#[path = "related/owner_selection.rs"]
pub mod owner_selection;

//! Target schema owns changed wire meaning and visibly changed production.
pub use super::{binding, limits, model, wire_model};
#[path = "cause.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod cause;
#[path = "derived_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod derived_query;
#[path = "../edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod edit;
#[path = "host.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod host;
#[path = "live_native.rs"]
pub mod live_native;
#[path = "live_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod live_query;
#[path = "membership.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the related membership declaration is compiled against the actual target schema"
)]
pub mod membership;
#[path = "successor/output.rs"]
pub mod output;
#[path = "page_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod page_query;
#[path = "paired_native.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the pair owner is called against the actual target bindings"
)]
pub mod paired_native;
#[path = "paired_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod paired_query;
#[path = "producer.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod producer;
#[path = "program.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod program;
#[path = "projection.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod projection;
#[path = "publish.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod publish;
#[path = "../../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod query;
#[path = "../readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod readiness;
#[path = "retire.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub mod retire;
#[path = "successor/schema.rs"]
pub mod schema;

pub fn heavy_value() -> model::NestedValue {
    super::journey::heavy()
}

#[path = "companion.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the identical companion owners are installed against the actual successor schema"
)]
pub mod companion;

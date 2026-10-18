//! Target schema owns changed wire meaning and visibly changed production.
use super::{binding, limits, model, wire_model};
#[path = "cause.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod cause;
#[path = "derived_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod derived_query;
#[path = "../edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod edit;
#[path = "host.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod host;
#[path = "live_native.rs"]
pub(super) mod live_native;
#[path = "live_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod live_query;
#[path = "membership.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the related membership declaration is compiled against the actual target schema"
)]
pub(super) mod membership;
#[path = "successor/output.rs"]
pub(super) mod output;
#[path = "page_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod page_query;
#[path = "paired_native.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the pair owner is called against the actual target bindings"
)]
pub(super) mod paired_native;
#[path = "paired_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod paired_query;
#[path = "producer.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod producer;
#[path = "program.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod program;
#[path = "projection.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod projection;
#[path = "publish.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod publish;
#[path = "../../canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod query;
#[path = "../readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod readiness;
#[path = "retire.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the same owners against its actual independent codec schema"
)]
pub(super) mod retire;
#[path = "successor/schema.rs"]
pub(super) mod schema;

fn heavy_value() -> model::NestedValue {
    super::journey::heavy()
}

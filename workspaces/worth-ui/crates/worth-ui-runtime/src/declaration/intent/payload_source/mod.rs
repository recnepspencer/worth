mod expression_resolution;
mod projection_resolution;
mod resolution;
mod resolved;

pub(crate) use resolution::resolve_payload_sources;
pub(crate) use resolved::{
    UiResolvedIntentApplicationSource, UiResolvedIntentPayloadBinding,
    UiResolvedIntentPayloadSource, UiResolvedIntentProjectionSource,
};

use super::{
    UiIntentApplicationFactPlan, UiIntentApplicationFactSlot, UiIntentCatalogPreparationDenial,
    UiIntentInteractionPayloadSourceKind, UiIntentSourcePlans, UiResolvedIntentExpressionSource,
};

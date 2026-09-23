mod equivalence;
mod generation_succession;
mod projection;
mod scroll_chrome_affordance;
mod snapshot;

pub(crate) use scroll_chrome_affordance::{
    resolve_scroll_chrome_pointer, UiScrollChromePointerAnswer, UiScrollChromeRegionTarget,
};

pub(crate) use generation_succession::UiPreparedPointerAffordanceGenerationSuccession;

pub(crate) use equivalence::UiPointerAffordanceReuseBasis;
pub(crate) use projection::UiPointerAffordanceProjection;
pub(crate) use snapshot::UiPointerAffordanceObservationIdentity;
pub(crate) use snapshot::UiPointerAffordanceSnapshot;

use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

use crate::memory_workspace::{
    WorthQueryMutationDelta, WorthQueryMutationKind, WorthQueryMutationReceipt,
};
use crate::program::WorthQueryDerivedView;

use super::WorthQueryAuthorityLane;

mod refresh_context;
mod routing;
mod state;
mod surface;

pub use refresh_context::{WorthQueryRetainedRefreshContext, WorthQueryRetainedRefreshOrigin};
pub(in crate::runtime) use routing::route_result::{
    admit_derived_view_declaration, insert_derived_runtime,
    retained_live_view_names_for_candidates, route_derived_view_patches,
};
pub(in crate::runtime) use state::{
    WorthQueryComputedAdmissionError, WorthQueryComputedDependencyIndex,
    WorthQueryDerivedViewRuntime,
};
pub use surface::{
    WorthQueryComputedInspectionEvidence, WorthQueryDerivedPatch, WorthQueryDerivedPatchFamily,
    WorthQueryDerivedPatchPayload, WorthQueryDerivedViewHandle, WorthQueryDerivedViewMaintainer,
    WorthQueryDerivedViewMaterialization, WorthQueryRetainedUpstreamInputs,
};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

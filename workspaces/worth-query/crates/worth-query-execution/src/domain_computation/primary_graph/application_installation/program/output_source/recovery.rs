//! Original discovered-source preparation held across unpublished World effects.

use std::any::{Any, TypeId};
use std::sync::Arc;

use worth_foundational::facade::AspectValue;

use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryPerformedRelationalProductChange, WorthQueryProductPublicationReceipt,
};
use crate::domain_computation::primary_graph::application_output_demand::{
    PreparedOutputRootKind, WorthQueryRequiredOutputSourcePreparation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

/// Move-only original preparation for an unpublished discovered source. Only
/// the program commit entrance creates it; a receipt cannot reconstruct it.
pub struct WorthQueryUnpublishedProgramOutputSource {
    pub(super) runtime_authority: u64,
    pub(super) program: AspectValue,
    pub(super) binding: TypeId,
    pub(super) idempotency: WorthQueryApplicationIdempotencyBinding,
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) root: PreparedOutputRootKind,
    pub(super) preparation: WorthQueryRequiredOutputSourcePreparation,
    pub(super) discovery: Arc<dyn Any + Send + Sync>,
}

/// Original performed carrier taken from the filled native terminal before
/// Query projection or history retirement. Missing parts remain owned and
/// refuse promotion; a partial transfer never drops an already-taken witness.
pub struct WorthQueryRecoveredProgramOutputSource {
    pub(super) publication: WorthQueryProductPublicationReceipt,
    pub(super) change: Option<Arc<WorthQueryPerformedRelationalProductChange>>,
    pub(super) observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

impl WorthQueryRecoveredProgramOutputSource {
    pub(in crate::domain_computation::primary_graph) fn take_original(
        publication: &WorthQueryProductPublicationReceipt,
    ) -> Self {
        Self {
            publication: publication.clone(),
            change: publication.take_fresh_delivery().map(Arc::new),
            observation: publication.take_output_demand_observation(),
        }
    }
}

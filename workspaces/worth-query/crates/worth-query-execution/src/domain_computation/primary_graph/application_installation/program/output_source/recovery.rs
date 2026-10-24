//! Original program-source preparation and one-use performed delivery.

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::sync::Arc;

use worth_foundational::facade::AspectValue;

use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryPerformedRelationalProductChange, WorthQueryProductPublicationReceipt,
};
use crate::domain_computation::primary_graph::application_output_demand::{
    PreparedOutputRootKind, WorthQueryRequiredOutputSourcePreparation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

pub(super) enum ProgramOutputSourcePayload {
    Required(TypeId),
    Discovered {
        root: TypeId,
        discovery: Arc<dyn Any + Send + Sync>,
    },
}

impl ProgramOutputSourcePayload {
    pub(super) fn root_kind(&self) -> PreparedOutputRootKind {
        match self {
            Self::Required(root) => PreparedOutputRootKind::Required(*root),
            Self::Discovered { root, .. } => PreparedOutputRootKind::Discovered(*root),
        }
    }

    pub(super) fn has_discovery<T: Any>(&self) -> bool {
        matches!(self, Self::Discovered { discovery, .. } if discovery.is::<T>())
    }

    pub(super) fn discovery(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        match self {
            Self::Required(_) => None,
            Self::Discovered { discovery, .. } => Some(Arc::clone(discovery)),
        }
    }
}

/// Move-only original preparation awaiting program-source retention. Only
/// the program commit entrance creates it; a receipt cannot reconstruct it.
pub struct WorthQueryUnpublishedProgramOutputSource {
    pub(super) correction: Option<UnpublishedCorrectionCommand>,
    pub(super) runtime_authority: u64,
    pub(super) program: AspectValue,
    pub(super) binding: TypeId,
    pub(super) idempotency: WorthQueryApplicationIdempotencyBinding,
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) payload: ProgramOutputSourcePayload,
    pub(super) preparation: WorthQueryRequiredOutputSourcePreparation,
}

pub(super) struct UnpublishedCorrectionCommand {
    intent: crate::domain_computation::primary_graph::WorthQueryCorrectionIntent,
    inverse_key_identity: [u8; 32],
}

impl UnpublishedCorrectionCommand {
    pub(super) fn new(
        intent: crate::domain_computation::primary_graph::WorthQueryCorrectionIntent,
        binding: &WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        Self {
            intent,
            inverse_key_identity: *binding.key_identity(),
        }
    }

    pub(super) fn bind(
        &self,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        if binding.key_identity() == &self.inverse_key_identity {
            self.intent.bind_idempotency(binding)
        } else {
            binding
        }
    }
}

impl WorthQueryUnpublishedProgramOutputSource {
    /// Extends a freshly admitted binding with this original correction command.
    /// Principal, input and scope continue to come from fresh ordinary admission.
    pub fn bind_recovery_idempotency(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        match &self.correction {
            Some(correction) => correction.bind(binding),
            None => binding,
        }
    }
}

/// Original performed carrier taken from the filled native terminal before
/// Query projection or history retirement. Missing parts remain owned and
/// refuse promotion; a partial transfer never drops an already-taken witness.
pub struct WorthQueryRecoveredProgramOutputSource {
    pub(super) publication:
        crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    pub(super) change: Option<Arc<WorthQueryPerformedRelationalProductChange>>,
    pub(super) observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

impl WorthQueryRecoveredProgramOutputSource {
    /// A source can be promoted only while both original one-shot parts remain.
    pub fn has_original_delivery(&self) -> bool {
        self.change.is_some() && self.observation.is_some()
    }

    pub(super) fn from_committed(
        receipt: &mut crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Self {
        let change = receipt
            .take_performed_relational_product_change()
            .map(Arc::new);
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation();
        Self {
            publication: receipt.committed_product_publication().clone(),
            change,
            observation,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn take_original(
        publication: &WorthQueryProductPublicationReceipt,
    ) -> Self {
        Self {
            publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication::from_receipt(publication.clone()),
            change: publication.take_fresh_delivery().map(Arc::new),
            observation: publication.take_output_demand_observation(),
        }
    }
}

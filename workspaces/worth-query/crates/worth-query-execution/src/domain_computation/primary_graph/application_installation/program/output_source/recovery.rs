//! Original program-source preparation and one-use performed delivery.

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::sync::Arc;

use worth_foundational::facade::AspectValue;

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
    pub(super) revision: worth_foundational::facade::CanonicalDigestId,
    pub(super) discovery_recipe: Option<Vec<u8>>,
    pub(super) root_identity: &'static str,
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
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;

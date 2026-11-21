//! A fresh key's conditional definition whose Bridge successor is not
//! installed at the selected product's Signal basis.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl WorthQueryApplicationCommitDenial {
    /// Checked only after the key resolves, so a retained original answers its
    /// key first; `detail()` carries Bridge's reason.
    pub(in crate::domain_computation::primary_graph) fn conditional_definition_basis(
        cause: &worth_runtime_bridge::facade::BridgeConditionalDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProductBasisStale,
            stage: WorthQueryApplicationCommitDenialStage::BridgePlanning,
            detail: Some(std::sync::Arc::from(cause.detail())),
            cause: None,
        }
    }
}

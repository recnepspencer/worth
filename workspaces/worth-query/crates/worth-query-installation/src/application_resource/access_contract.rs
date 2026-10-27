use super::WorthQueryResourceAccessPurpose;
use crate::graph_obligation::{
    WorthQueryInstalledGraphObligationInspection, WorthQueryInstalledGraphObligationSet,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureRule;

/// Immutable installed action/output correspondence. Construction is confined
/// to the actual typed catalog; this grants neither permission nor capacity.
pub struct WorthQueryInstalledResourceAccessContract {
    pub(super) purpose: WorthQueryResourceAccessPurpose,
    pub(super) obligations: Arc<WorthQueryInstalledGraphObligationSet>,
    pub(super) rules: Vec<ApplicationQueryDisclosureRule>,
}

impl WorthQueryInstalledResourceAccessContract {
    pub const fn purpose(&self) -> WorthQueryResourceAccessPurpose {
        self.purpose
    }
    pub fn graph_obligations(&self) -> WorthQueryInstalledGraphObligationInspection<'_> {
        self.obligations.inspect()
    }
    pub fn disclosure_rules(&self) -> &[ApplicationQueryDisclosureRule] {
        &self.rules
    }
    /// Integration custody of the actual compiled set; no reconstruction.
    #[doc(hidden)]
    pub fn retain_graph_obligations_for_admission(
        &self,
    ) -> Arc<WorthQueryInstalledGraphObligationSet> {
        Arc::clone(&self.obligations)
    }
}

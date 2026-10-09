//! Affinity of an existing root-tagged token after source consumption.
use super::*;
impl WorthQueryOutputDemandRegistry {
    /// Validates an already owned token throughout root/dependent progression.
    /// Completion may consume the source carrier while this token still holds
    /// the original root-tagged registry row; it does not re-admit a source.
    pub(in crate::domain_computation::primary_graph) fn validate_owned_root_kind(
        &self,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        kind: PreparedOutputRootKind,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = state
            .source_custody
            .get(&prepared.source_commit)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "owned program source has no retained registry row",
                )
            })?;
        if let Some(retired) = &custody.retired {
            return Err(retired.clone());
        }
        if custody.root_kind != kind || custody.occurrence != prepared.product_occurrence {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "owned program source differs from its original root or occurrence",
            ));
        }
        Ok(())
    }
}

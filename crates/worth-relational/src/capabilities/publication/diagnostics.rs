use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn publication_diagnostics(&self) -> Vec<RelationalDiagnosticArtifact> {
        self.history
            .performed_canonical_envelopes()
            .into_iter()
            .map(|envelope| envelope.diagnostics_summary.clone())
            .filter(|artifact| !artifact.entries.is_empty())
            .collect()
    }

    pub(crate) fn publication_diagnostic_artifact_count(&self) -> usize {
        self.publication_diagnostics().len()
    }

    pub(crate) fn publication_diagnostics_since(
        &self,
        start: usize,
    ) -> Vec<RelationalDiagnosticArtifact> {
        self.publication_diagnostics()
            .into_iter()
            .skip(start)
            .collect()
    }
}

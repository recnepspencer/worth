use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::runtime::RelationalRuntime;

pub(crate) trait PublicationDiagnosticsSource {
    fn publication_diagnostics(&self) -> Vec<RelationalDiagnosticArtifact>;
    fn publication_diagnostic_artifact_count(&self) -> usize;
    fn publication_diagnostics_since(&self, start: usize) -> Vec<RelationalDiagnosticArtifact>;
}

impl PublicationDiagnosticsSource for RelationalRuntime {
    fn publication_diagnostics(&self) -> Vec<RelationalDiagnosticArtifact> {
        self.history
            .performed_canonical_envelopes()
            .into_iter()
            .map(|envelope| envelope.diagnostics_summary.clone())
            .filter(|artifact| !artifact.entries.is_empty())
            .collect()
    }

    fn publication_diagnostic_artifact_count(&self) -> usize {
        self.publication_diagnostics().len()
    }

    fn publication_diagnostics_since(&self, start: usize) -> Vec<RelationalDiagnosticArtifact> {
        self.publication_diagnostics().into_iter().skip(start).collect()
    }
}

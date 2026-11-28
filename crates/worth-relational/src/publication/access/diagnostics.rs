use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::diagnostics::data::RelationalDiagnosticsFacade;

use super::PublicationDiagnosticsAccess;

impl<'runtime> PublicationDiagnosticsAccess<'runtime> {
    pub fn facade(&self) -> RelationalDiagnosticsFacade {
        RelationalDiagnosticsFacade {
            artifacts: self.runtime.publication_diagnostics(),
        }
    }

    pub fn artifacts(&self) -> Vec<RelationalDiagnosticArtifact> {
        self.runtime.publication_diagnostics()
    }

    pub fn artifact_count(&self) -> usize {
        self.runtime.publication_diagnostic_artifact_count()
    }

    pub fn artifacts_since(&self, start: usize) -> Vec<RelationalDiagnosticArtifact> {
        self.runtime.publication_diagnostics_since(start)
    }
}

use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticFields, RelationalDiagnosticsEntry,
};
use crate::publication::PublicationAuthority;
use crate::runtime::RelationalRuntime;

pub(crate) struct DiagnosticArtifactBuilder<'runtime> {
    runtime: &'runtime RelationalRuntime,
    scope: DiagnosticsScope,
    kind: DiagnosticsArtifactKind,
    entries: Vec<RelationalDiagnosticsEntry>,
}

fn emit_filtered_artifact(
    runtime: &RelationalRuntime,
    artifact: RelationalDiagnosticArtifact,
) -> RelationalDiagnosticArtifact {
    let profile = &runtime.config.diagnostics.profile;
    let filtered = profile
        .filter_artifact(artifact.clone())
        .unwrap_or_else(|| {
            RelationalDiagnosticArtifact::new(
                artifact.scope,
                artifact.kind,
                artifact.determinism,
                Vec::new(),
            )
        });
    if !filtered.entries.is_empty() {
        runtime.publication.diagnostics.push(filtered.clone());
    }
    filtered
}

impl<'runtime> DiagnosticArtifactBuilder<'runtime> {
    fn new(runtime: &'runtime RelationalRuntime, scope: DiagnosticsScope) -> Self {
        Self {
            runtime,
            scope,
            kind: DiagnosticsArtifactKind::MinimalSummary,
            entries: Vec::new(),
        }
    }

    pub(crate) fn kind(mut self, kind: DiagnosticsArtifactKind) -> Self {
        self.kind = kind;
        self
    }

    pub(crate) fn minimal_summary(self) -> Self {
        self.kind(DiagnosticsArtifactKind::MinimalSummary)
    }

    pub(crate) fn failure(self) -> Self {
        self.kind(DiagnosticsArtifactKind::Failure)
    }

    pub(crate) fn comparison(self) -> Self {
        self.kind(DiagnosticsArtifactKind::Comparison)
    }

    pub(crate) fn entry(
        mut self,
        code: crate::diagnostics::data::DiagnosticCode,
        message: impl Into<String>,
        fields: impl Into<RelationalDiagnosticFields>,
    ) -> Self {
        self.entries.push(RelationalDiagnosticsEntry::new(
            code,
            message,
            fields.into(),
        ));
        self
    }

    pub(crate) fn entries(
        mut self,
        entries: impl IntoIterator<Item = RelationalDiagnosticsEntry>,
    ) -> Self {
        self.entries.extend(entries);
        self
    }

    pub(crate) fn emit_entry(
        self,
        code: crate::diagnostics::data::DiagnosticCode,
        message: impl Into<String>,
        fields: impl Into<RelationalDiagnosticFields>,
    ) -> RelationalDiagnosticArtifact {
        self.entry(code, message, fields).emit()
    }

    pub(crate) fn emit(self) -> RelationalDiagnosticArtifact {
        emit_filtered_artifact(
            self.runtime,
            RelationalDiagnosticArtifact::new(
                self.scope,
                self.kind,
                DeterminismExpectation::Required,
                self.entries,
            ),
        )
    }
}

impl RelationalRuntime {
    pub(crate) fn push_preparation_diagnostic_artifact(
        &self,
        artifact: RelationalDiagnosticArtifact,
    ) {
        let _ = emit_filtered_artifact(self, artifact);
    }

    pub(crate) fn push_bounded_preparation_diagnostic(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
        entries: Vec<RelationalDiagnosticsEntry>,
    ) -> RelationalDiagnosticArtifact {
        emit_filtered_artifact(
            self,
            RelationalDiagnosticArtifact::new(
                scope,
                kind,
                DeterminismExpectation::Required,
                entries,
            ),
        )
    }
}

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn push_diagnostic_artifact(&self, artifact: RelationalDiagnosticArtifact) {
        let _ = emit_filtered_artifact(self.runtime, artifact);
    }

    pub(crate) fn push_bounded_diagnostic(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
        entries: Vec<RelationalDiagnosticsEntry>,
    ) -> RelationalDiagnosticArtifact {
        emit_filtered_artifact(
            self.runtime,
            RelationalDiagnosticArtifact::new(
                scope,
                kind,
                DeterminismExpectation::Required,
                entries,
            ),
        )
    }

    pub(crate) fn diagnostic(self, scope: DiagnosticsScope) -> DiagnosticArtifactBuilder<'runtime> {
        DiagnosticArtifactBuilder::new(self.runtime, scope)
    }
}

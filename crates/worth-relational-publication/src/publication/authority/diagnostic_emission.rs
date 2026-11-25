use crate::publication::authority::PublicationAuthority;
use crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore;
use contracts::diagnostics::data::artifacts::DeterminismExpectation;
use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::diagnostics::data::fields::RelationalDiagnosticFields;
use contracts::diagnostics::data::profiles::RelationalDiagnosticsProfile;
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone, Copy)]
pub struct PreparationDiagnostics<'a> {
    pub profile: &'a RelationalDiagnosticsProfile,
    pub store: &'a RelationalDiagnosticArtifactStore,
    pub instrumentation:
        &'a worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
}

impl PreparationDiagnostics<'_> {
    pub fn emit(
        self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
        entries: Vec<RelationalDiagnosticsEntry>,
    ) -> RelationalDiagnosticArtifact {
        emit_bounded_artifact(self.profile, self.store, scope, kind, entries)
    }

    pub(crate) fn emit_failure_diagnostic(
        self,
        scope: DiagnosticsScope,
        code: DiagnosticCode,
        message: impl Into<String>,
        fields: impl Into<RelationalDiagnosticFields>,
    ) {
        DiagnosticArtifactBuilder::new(self.profile, self.store, scope)
            .failure()
            .emit_entry(code, message, fields);
    }
}

pub struct DiagnosticArtifactBuilder<'runtime> {
    profile: &'runtime RelationalDiagnosticsProfile,
    store: &'runtime RelationalDiagnosticArtifactStore,
    scope: DiagnosticsScope,
    kind: DiagnosticsArtifactKind,
    entries: Vec<RelationalDiagnosticsEntry>,
}

pub(crate) fn emit_filtered_artifact(
    profile: &RelationalDiagnosticsProfile,
    store: &RelationalDiagnosticArtifactStore,
    artifact: RelationalDiagnosticArtifact,
) -> RelationalDiagnosticArtifact {
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
        store.push(filtered.clone());
    }
    filtered
}

pub fn emit_bounded_artifact(
    profile: &RelationalDiagnosticsProfile,
    store: &RelationalDiagnosticArtifactStore,
    scope: DiagnosticsScope,
    kind: DiagnosticsArtifactKind,
    entries: Vec<RelationalDiagnosticsEntry>,
) -> RelationalDiagnosticArtifact {
    emit_filtered_artifact(
        profile,
        store,
        RelationalDiagnosticArtifact::new(scope, kind, DeterminismExpectation::Required, entries),
    )
}

impl<'runtime> DiagnosticArtifactBuilder<'runtime> {
    pub(crate) fn new(
        profile: &'runtime RelationalDiagnosticsProfile,
        store: &'runtime RelationalDiagnosticArtifactStore,
        scope: DiagnosticsScope,
    ) -> Self {
        Self {
            profile,
            store,
            scope,
            kind: DiagnosticsArtifactKind::MinimalSummary,
            entries: Vec::new(),
        }
    }

    pub(crate) fn kind(mut self, kind: DiagnosticsArtifactKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn minimal_summary(self) -> Self {
        self.kind(DiagnosticsArtifactKind::MinimalSummary)
    }

    pub fn failure(self) -> Self {
        self.kind(DiagnosticsArtifactKind::Failure)
    }

    pub fn comparison(self) -> Self {
        self.kind(DiagnosticsArtifactKind::Comparison)
    }

    pub(crate) fn entry(
        mut self,
        code: contracts::diagnostics::data::artifacts::DiagnosticCode,
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

    pub fn entries(
        mut self,
        entries: impl IntoIterator<Item = RelationalDiagnosticsEntry>,
    ) -> Self {
        self.entries.extend(entries);
        self
    }

    pub fn emit_entry(
        self,
        code: contracts::diagnostics::data::artifacts::DiagnosticCode,
        message: impl Into<String>,
        fields: impl Into<RelationalDiagnosticFields>,
    ) -> RelationalDiagnosticArtifact {
        self.entry(code, message, fields).emit()
    }

    pub fn emit(self) -> RelationalDiagnosticArtifact {
        emit_bounded_artifact(
            self.profile,
            self.store,
            self.scope,
            self.kind,
            self.entries,
        )
    }
}

impl<'runtime> PublicationAuthority<'runtime> {
    pub fn preparation_diagnostics(&self) -> PreparationDiagnostics<'runtime> {
        PreparationDiagnostics {
            profile: self.diagnostics_profile,
            store: &self.publication.diagnostics,
            instrumentation: self.instrumentation,
        }
    }

    pub fn push_diagnostic_artifact(&self, artifact: RelationalDiagnosticArtifact) {
        let _ = emit_filtered_artifact(
            self.diagnostics_profile,
            &self.publication.diagnostics,
            artifact,
        );
    }

    pub fn push_bounded_diagnostic(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
        entries: Vec<RelationalDiagnosticsEntry>,
    ) -> RelationalDiagnosticArtifact {
        emit_bounded_artifact(
            self.diagnostics_profile,
            &self.publication.diagnostics,
            scope,
            kind,
            entries,
        )
    }

    pub fn diagnostic(self, scope: DiagnosticsScope) -> DiagnosticArtifactBuilder<'runtime> {
        DiagnosticArtifactBuilder::new(
            self.diagnostics_profile,
            &self.publication.diagnostics,
            scope,
        )
    }
}

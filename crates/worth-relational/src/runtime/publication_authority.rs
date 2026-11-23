//! Bind publication services to the installed runtime or its preparation snapshot.
use super::RelationalRuntime;
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::mvcc::publication::authority::{
    RelationalPreparationPublicationAuthority, RelationalPublicationAuthority,
};
use crate::publication::authority::diagnostic_emission::emit_filtered_artifact;
use crate::publication::{PublicationAuthority, PublicationPreparationAuthority};

impl RelationalRuntime {
    pub fn publication_companion_port(&self) -> crate::mvcc::PublicationCompanionRegistrationPort {
        crate::mvcc::PublicationCompanionRegistrationPort::new(
            self.runtime_instance_id(),
            self.owner_binding(),
            self.publication_binding(),
        )
    }
}

impl RelationalRuntime {
    pub(crate) fn publication_authority(&self) -> PublicationAuthority<'_> {
        PublicationAuthority::new(
            &self.publication,
            &self.visibility,
            &self.config.diagnostics.profile,
            &self.services.instrumentation,
        )
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn publication_preparation_authority(&self) -> PublicationPreparationAuthority<'_> {
        PublicationPreparationAuthority::new(
            self.runtime_instance_id(),
            self.snapshot_id_allocator(),
        )
    }
}

impl RelationalRuntime {
    pub(crate) fn mvcc_publication_authority(&self) -> RelationalPublicationAuthority<'_> {
        RelationalPublicationAuthority { runtime: self }
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn mvcc_publication_authority(
        &self,
    ) -> RelationalPreparationPublicationAuthority<'_> {
        RelationalPreparationPublicationAuthority { runtime: self }
    }
}

impl RelationalRuntime {
    pub(crate) fn push_preparation_diagnostic_artifact(
        &self,
        artifact: RelationalDiagnosticArtifact,
    ) {
        let _ = emit_filtered_artifact(
            &self.config.diagnostics.profile,
            &self.publication.diagnostics,
            artifact,
        );
    }

    pub(crate) fn push_bounded_preparation_diagnostic(
        &self,
        scope: DiagnosticsScope,
        kind: DiagnosticsArtifactKind,
        entries: Vec<RelationalDiagnosticsEntry>,
    ) -> RelationalDiagnosticArtifact {
        emit_filtered_artifact(
            &self.config.diagnostics.profile,
            &self.publication.diagnostics,
            RelationalDiagnosticArtifact::new(
                scope,
                kind,
                DeterminismExpectation::Required,
                entries,
            ),
        )
    }
}

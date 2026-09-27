use std::cmp::Ordering;
use worth_ui_dsl::{WorthUiArtifactInputProvenance, WorthUiSourceModuleId};

use crate::capability::CapabilityRegistrationDiagnostic;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum WorthUiResolutionDiagnosticCode {
    InvalidComponentReferenceId,
    MissingComponentReference,
    /// Registration named the capability but validation refused it.
    RejectedComponentReference,
    DeferredComponentReference,
    UnsupportedComponentReference,
    PlatformInternalComponentReference,
    InvalidSurfaceReferenceId,
    MissingSurfaceReference,
    /// Registration named the capability but validation refused it.
    RejectedSurfaceReference,
    DeferredSurfaceReference,
    UnsupportedSurfaceReference,
    PlatformInternalSurfaceReference,
    InvalidViewBindingReferenceId,
    MissingViewBindingReference,
    /// Registration named the capability but validation refused it.
    RejectedViewBindingReference,
    DeferredViewBindingReference,
    UnsupportedViewBindingReference,
    PlatformInternalViewBindingReference,
    InvalidThemeTokenReferenceId,
    MissingThemeTokenReference,
    /// Registration named the capability but validation refused it.
    RejectedThemeTokenReference,
    DeferredThemeTokenReference,
    UnsupportedThemeTokenReference,
    PlatformInternalThemeTokenReference,
}

impl WorthUiResolutionDiagnosticCode {
    const fn is_missing_reference(self) -> bool {
        matches!(
            self,
            Self::MissingComponentReference
                | Self::MissingSurfaceReference
                | Self::MissingViewBindingReference
                | Self::MissingThemeTokenReference
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiResolutionDiagnostic {
    code: WorthUiResolutionDiagnosticCode,
    module_id: WorthUiSourceModuleId,
    authored_text: String,
    provenance: Box<WorthUiArtifactInputProvenance>,
    /// What refused the referenced capability's registration, for a
    /// `Rejected*Reference`.
    registration: Box<[CapabilityRegistrationDiagnostic]>,
}

impl WorthUiResolutionDiagnostic {
    pub(crate) fn new(
        code: WorthUiResolutionDiagnosticCode,
        module_id: WorthUiSourceModuleId,
        authored_text: impl Into<String>,
        provenance: WorthUiArtifactInputProvenance,
    ) -> Self {
        Self {
            code,
            module_id,
            authored_text: authored_text.into(),
            provenance: Box::new(provenance),
            registration: Box::default(),
        }
    }

    /// Carries the diagnostics that refused the referenced capability's
    /// registration, when validation refused it, and reports a missing
    /// reference as `rejected` instead. A reference whose support catalog
    /// entry declares a posture (deferred, unsupported or platform internal)
    /// keeps that code, since the declared posture is what the author must
    /// act on; the registration diagnostics still travel with it.
    pub(crate) fn refused_at_registration(
        mut self,
        rejected: WorthUiResolutionDiagnosticCode,
        registration: Option<&[CapabilityRegistrationDiagnostic]>,
    ) -> Self {
        if let Some(registration) = registration {
            if self.code.is_missing_reference() {
                self.code = rejected;
            }
            self.registration = registration.into();
        }
        self
    }

    #[cfg(test)]
    pub(crate) fn registration(&self) -> &[CapabilityRegistrationDiagnostic] {
        &self.registration
    }

    #[cfg(test)]
    pub(crate) fn code(&self) -> WorthUiResolutionDiagnosticCode {
        self.code
    }

    #[cfg(test)]
    pub(crate) fn authored_text(&self) -> &str {
        &self.authored_text
    }

    #[cfg(test)]
    pub(crate) fn module_id(&self) -> &WorthUiSourceModuleId {
        &self.module_id
    }

    #[cfg(test)]
    pub(crate) fn provenance(&self) -> &WorthUiArtifactInputProvenance {
        &self.provenance
    }

    pub(crate) fn stable_cmp(&self, other: &Self) -> Ordering {
        self.code
            .cmp(&other.code)
            .then_with(|| self.module_id.cmp(&other.module_id))
            .then_with(|| self.authored_text.cmp(&other.authored_text))
            .then_with(|| stable_provenance_cmp(&self.provenance, &other.provenance))
    }
}

fn stable_provenance_cmp(
    left: &WorthUiArtifactInputProvenance,
    right: &WorthUiArtifactInputProvenance,
) -> Ordering {
    match (left, right) {
        (
            WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
                declaration_span: left_declaration,
                detail_span: left_detail,
                declaration_index: left_index,
            },
            WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
                declaration_span: right_declaration,
                detail_span: right_detail,
                declaration_index: right_index,
            },
        ) => stable_span_cmp(left_declaration, right_declaration)
            .then_with(|| stable_optional_span_cmp(left_detail.as_ref(), right_detail.as_ref()))
            .then_with(|| left_index.cmp(right_index)),
        (
            WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
                authored_module_path: left_path,
                declaration_index: left_index,
            },
            WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
                authored_module_path: right_path,
                declaration_index: right_index,
            },
        ) => left_path
            .cmp(right_path)
            .then_with(|| left_index.cmp(right_index)),
        (WorthUiArtifactInputProvenance::ParsedSourceDeclaration { .. }, _) => Ordering::Less,
        (_, WorthUiArtifactInputProvenance::ParsedSourceDeclaration { .. }) => Ordering::Greater,
    }
}

fn stable_optional_span_cmp(
    left: Option<&worth_ui_dsl::WorthUiSourceSpan>,
    right: Option<&worth_ui_dsl::WorthUiSourceSpan>,
) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => stable_span_cmp(left, right),
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn stable_span_cmp(
    left: &worth_ui_dsl::WorthUiSourceSpan,
    right: &worth_ui_dsl::WorthUiSourceSpan,
) -> Ordering {
    left.module_id()
        .cmp(right.module_id())
        .then_with(|| left.start_byte().cmp(&right.start_byte()))
        .then_with(|| left.end_byte().cmp(&right.end_byte()))
}

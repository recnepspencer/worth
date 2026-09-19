use crate::capability::{CapabilityDiagnosticCode, CapabilitySupportKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegistrationCandidateDiagnostic {
    code: CapabilityDiagnosticCode,
    detail: &'static str,
}

impl RegistrationCandidateDiagnostic {
    pub(crate) fn new(code: CapabilityDiagnosticCode, detail: &'static str) -> Self {
        Self { code, detail }
    }

    pub(crate) fn code(&self) -> CapabilityDiagnosticCode {
        self.code
    }

    pub(crate) fn detail(&self) -> &'static str {
        self.detail
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegistrationDependency {
    expected_family_name: &'static str,
    actual_family_name: &'static str,
    identity_text: String,
}

impl RegistrationDependency {
    pub(crate) fn new(
        expected_family_name: &'static str,
        actual_family_name: &'static str,
        identity_text: impl Into<String>,
    ) -> Self {
        Self {
            expected_family_name,
            actual_family_name,
            identity_text: identity_text.into(),
        }
    }

    pub(crate) fn expected_family_name(&self) -> &'static str {
        self.expected_family_name
    }

    pub(crate) fn actual_family_name(&self) -> &'static str {
        self.actual_family_name
    }

    pub(crate) fn identity_text(&self) -> &str {
        &self.identity_text
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegistrationCandidate {
    family_name: &'static str,
    identity_text: String,
    support_kind: CapabilitySupportKind,
    dependencies: Vec<RegistrationDependency>,
    descriptor_diagnostics: Vec<RegistrationCandidateDiagnostic>,
}

impl RegistrationCandidate {
    #[cfg(test)]
    pub(crate) fn admitted(family_name: &'static str, identity_text: impl Into<String>) -> Self {
        Self::new(family_name, identity_text, CapabilitySupportKind::Admitted)
    }

    #[cfg(test)]
    pub(crate) fn with_support(
        family_name: &'static str,
        identity_text: impl Into<String>,
        support_kind: CapabilitySupportKind,
    ) -> Self {
        Self::new(family_name, identity_text, support_kind)
    }

    pub(crate) fn with_dependency(mut self, dependency: RegistrationDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub(crate) fn with_descriptor_diagnostic(
        mut self,
        diagnostic: RegistrationCandidateDiagnostic,
    ) -> Self {
        self.descriptor_diagnostics.push(diagnostic);
        self
    }

    pub(crate) fn record_descriptor_diagnostic(
        &mut self,
        diagnostic: RegistrationCandidateDiagnostic,
    ) {
        self.descriptor_diagnostics.push(diagnostic);
    }

    pub(crate) fn family_name(&self) -> &'static str {
        self.family_name
    }

    pub(crate) fn identity_text(&self) -> &str {
        &self.identity_text
    }

    pub(crate) fn support_kind(&self) -> CapabilitySupportKind {
        self.support_kind
    }

    pub(crate) fn dependencies(&self) -> &[RegistrationDependency] {
        &self.dependencies
    }

    pub(crate) fn descriptor_diagnostics(&self) -> &[RegistrationCandidateDiagnostic] {
        &self.descriptor_diagnostics
    }

    pub(crate) fn new(
        family_name: &'static str,
        identity_text: impl Into<String>,
        support_kind: CapabilitySupportKind,
    ) -> Self {
        Self {
            family_name,
            identity_text: identity_text.into(),
            support_kind,
            dependencies: Vec::new(),
            descriptor_diagnostics: Vec::new(),
        }
    }
}

use crate::capability::{
    CapabilityDiagnosticCode, CapabilitySupportKind, RegistrationCandidate,
    RegistrationCandidateDiagnostic, RegistrationDependency, THEME_TOKEN_FAMILY_NAME,
};

use super::super::{ThemeTokenDescriptor, ThemeTokenSource};

impl ThemeTokenDescriptor {
    pub(crate) fn registration_candidate(&self) -> RegistrationCandidate {
        let candidate = RegistrationCandidate::new(
            THEME_TOKEN_FAMILY_NAME,
            self.id().as_str(),
            CapabilitySupportKind::Admitted,
        );
        add_theme_token_dependencies(add_theme_token_diagnostics(candidate, self), self)
    }
}

fn add_theme_token_diagnostics(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    candidate = add_unknown_family_diagnostic(candidate, descriptor);
    candidate = add_missing_definition_diagnostic(candidate, descriptor);
    candidate = add_invalid_value_diagnostic(candidate, descriptor);
    candidate = add_raw_color_diagnostic(candidate, descriptor);
    candidate = add_plugin_platform_override_diagnostic(candidate, descriptor);
    add_plugin_contribution_kind_diagnostic(candidate, descriptor)
}

fn add_theme_token_dependencies(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    if let Some(alias_target) = descriptor.alias_target() {
        candidate = candidate.with_dependency(RegistrationDependency::new(
            THEME_TOKEN_FAMILY_NAME,
            THEME_TOKEN_FAMILY_NAME,
            alias_target.as_str(),
        ));
    }
    candidate
}

fn add_unknown_family_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    if !descriptor.family().is_known() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::UnknownThemeTokenFamily,
            "theme token family must be a built-in domain-agnostic presentation family",
        );
    }
    candidate
}

fn add_missing_definition_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    if !descriptor.has_definition() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingThemeTokenDefinition,
            "theme token must define exactly one value or alias",
        );
    }
    candidate
}

fn add_invalid_value_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    if descriptor.value().is_some_and(|value| !value.is_valid()) {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::InvalidThemeTokenValue,
            "theme token color values must be explicit #RRGGBB or #RRGGBBAA literals",
        );
    }
    candidate
}

fn add_raw_color_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    if descriptor.has_raw_color_outside_token_definition() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::RawColorOutsideThemeTokenDefinition,
            "raw color literals must be wrapped in a named theme token definition",
        );
    }
    candidate
}

fn add_plugin_platform_override_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    if descriptor.source().claims_platform_override()
        || plugin_claims_platform_token_identity(descriptor)
    {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::PluginThemeTokenOverridesPlatformMeaning,
            "plugins cannot silently redefine platform theme token meaning",
        );
    }
    candidate
}

fn plugin_claims_platform_token_identity(descriptor: &ThemeTokenDescriptor) -> bool {
    descriptor.source().is_plugin_contribution()
        && token_identity_has_platform_segment(descriptor.id().as_str())
}

fn token_identity_has_platform_segment(identity_text: &str) -> bool {
    identity_text
        .split('.')
        .any(|segment| segment == "platform")
}

fn add_plugin_contribution_kind_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &ThemeTokenDescriptor,
) -> RegistrationCandidate {
    let mismatch = match descriptor.source() {
        ThemeTokenSource::PluginCustom => descriptor.value().is_none(),
        ThemeTokenSource::PluginAlias => descriptor.alias_definition().is_none(),
        ThemeTokenSource::Platform
        | ThemeTokenSource::Application
        | ThemeTokenSource::PluginPlatformOverride => false,
    };

    if mismatch {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::PluginThemeTokenContributionKindMismatch,
            "plugin theme token contribution source must match custom value or alias shape",
        );
    }
    candidate
}

fn with_descriptor_diagnostic(
    candidate: RegistrationCandidate,
    code: CapabilityDiagnosticCode,
    message: &'static str,
) -> RegistrationCandidate {
    candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(code, message))
}

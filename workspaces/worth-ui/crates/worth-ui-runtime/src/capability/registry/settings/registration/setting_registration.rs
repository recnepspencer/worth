use crate::capability::{
    CapabilityDiagnosticCode, CapabilitySupportKind, RegistrationCandidate,
    RegistrationCandidateDiagnostic, SETTING_FAMILY_NAME,
};

use super::super::SettingDescriptor;

impl SettingDescriptor {
    pub(crate) fn registration_candidate(&self) -> RegistrationCandidate {
        let candidate = RegistrationCandidate::new(
            SETTING_FAMILY_NAME,
            self.id().as_str(),
            CapabilitySupportKind::Admitted,
        );
        add_setting_diagnostics(candidate, self)
    }
}

fn add_setting_diagnostics(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    candidate = add_arbitrary_key_value_bag_diagnostic(candidate, descriptor);
    candidate = add_missing_scope_diagnostic(candidate, descriptor);
    candidate = add_missing_value_schema_diagnostic(candidate, descriptor);
    candidate = add_missing_default_posture_diagnostic(candidate, descriptor);
    candidate = add_missing_validation_posture_diagnostic(candidate, descriptor);
    candidate = add_missing_migration_posture_diagnostic(candidate, descriptor);
    candidate = add_missing_editor_hint_diagnostic(candidate, descriptor);
    candidate = add_missing_ownership_metadata_diagnostic(candidate, descriptor);
    candidate = add_invalid_value_schema_diagnostic(candidate, descriptor);
    candidate = add_default_value_schema_mismatch_diagnostic(candidate, descriptor);
    add_domain_truth_claim_diagnostic(candidate, descriptor)
}

fn add_arbitrary_key_value_bag_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.has_arbitrary_key_value_bag() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::ArbitraryKeyValueSettingBag,
            "settings must be typed descriptors, not arbitrary key/value bags",
        );
    }
    candidate
}

fn add_missing_scope_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.scope().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingScope,
            "settings must declare structural scope ownership",
        );
    }
    candidate
}

fn add_missing_value_schema_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.value_schema().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingValueSchema,
            "settings must declare a typed value schema",
        );
    }
    candidate
}

fn add_missing_default_posture_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.default_posture().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingDefaultPosture,
            "settings must declare default posture",
        );
    }
    candidate
}

fn add_missing_validation_posture_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.validation_posture().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingValidationPosture,
            "settings must declare validation posture",
        );
    }
    candidate
}

fn add_missing_migration_posture_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.migration_posture().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingMigrationPosture,
            "settings must declare migration posture even when migration artifacts are deferred",
        );
    }
    candidate
}

fn add_missing_editor_hint_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.editor_hint().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingEditorHint,
            "settings must declare the editor hint used by later settings surfaces",
        );
    }
    candidate
}

fn add_missing_ownership_metadata_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor.ownership_metadata().is_none() {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::MissingSettingOwnershipMetadata,
            "settings must declare ownership metadata without claiming domain truth",
        );
    }
    candidate
}

fn add_invalid_value_schema_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if descriptor
        .value_schema()
        .is_some_and(|schema| !schema.is_valid_schema())
    {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::InvalidSettingValueSchema,
            "setting value schema must be structurally valid and deterministic",
        );
    }
    candidate
}

fn add_default_value_schema_mismatch_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    if let (Some(schema), Some(default_posture)) =
        (descriptor.value_schema(), descriptor.default_posture())
        && default_posture
            .default_value()
            .is_some_and(|value| !schema.admits_default_value(value))
    {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::SettingDefaultValueSchemaMismatch,
            "setting default value must satisfy the declared value schema",
        );
    }
    candidate
}

fn add_domain_truth_claim_diagnostic(
    mut candidate: RegistrationCandidate,
    descriptor: &SettingDescriptor,
) -> RegistrationCandidate {
    let migration_claims_domain_truth = descriptor
        .migration_posture()
        .is_some_and(|posture| posture.claims_domain_truth());
    let ownership_claims_domain_truth = descriptor
        .ownership_metadata()
        .is_some_and(|metadata| metadata.claims_domain_truth());

    if migration_claims_domain_truth || ownership_claims_domain_truth {
        candidate = with_descriptor_diagnostic(
            candidate,
            CapabilityDiagnosticCode::SettingPersistenceClaimsDomainTruth,
            "settings metadata and migration posture cannot claim authoritative domain truth",
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

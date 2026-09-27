use std::collections::BTreeSet;

use crate::capability::{
    CapabilityRegistrationDiagnostic, RegisteredCapabilitySet, RegistryFamily,
};

pub(crate) type AcceptedRegistrationKey = (&'static str, String);

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RegistrationValidationReport {
    accepted_capabilities: RegisteredCapabilitySet,
    accepted_registration_keys: BTreeSet<AcceptedRegistrationKey>,
    diagnostics: Vec<CapabilityRegistrationDiagnostic>,
}

impl RegistrationValidationReport {
    pub(crate) fn new(
        accepted_capabilities: RegisteredCapabilitySet,
        accepted_registration_keys: BTreeSet<AcceptedRegistrationKey>,
        diagnostics: Vec<CapabilityRegistrationDiagnostic>,
    ) -> Self {
        Self {
            accepted_capabilities,
            accepted_registration_keys,
            diagnostics,
        }
    }

    #[cfg(test)]
    pub(crate) fn accepted_capabilities(&self) -> &RegisteredCapabilitySet {
        &self.accepted_capabilities
    }

    #[cfg(test)]
    pub(crate) fn diagnostics(&self) -> &[CapabilityRegistrationDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn accepts(&self, family_name: &'static str, identity_text: &str) -> bool {
        self.accepted_registration_keys
            .contains(&(family_name, identity_text.to_owned()))
    }

    /// The diagnostics validation recorded against `identity_text` in
    /// `family_name`.
    pub(crate) fn diagnostics_of<'report>(
        &'report self,
        family_name: &'static str,
        identity_text: &'report str,
    ) -> impl Iterator<Item = &'report CapabilityRegistrationDiagnostic> {
        self.diagnostics.iter().filter(move |diagnostic| {
            diagnostic.family_name() == Some(family_name)
                && diagnostic.identity_text() == Some(identity_text)
        })
    }

    pub(crate) fn accepted_identity_texts_for_family(
        &self,
        family_name: &'static str,
    ) -> BTreeSet<String> {
        self.accepted_registration_keys
            .iter()
            .filter(|(accepted_family_name, _)| *accepted_family_name == family_name)
            .map(|(_, identity_text)| identity_text.clone())
            .collect()
    }

    pub(crate) fn accepted_identity_texts_for_registry_family(
        &self,
        registry_family: RegistryFamily,
    ) -> BTreeSet<String> {
        self.accepted_identity_texts_for_family(registry_family.name())
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RegisteredCapabilitySet,
        Vec<CapabilityRegistrationDiagnostic>,
    ) {
        (self.accepted_capabilities, self.diagnostics)
    }
}

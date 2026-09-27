use std::collections::BTreeMap;

use crate::capability::{
    CapabilityRegistrationDiagnostic, RegistrationCandidate, RegistrationValidationReport,
};

/// Registrations validation refused, each with the diagnostics that refused
/// it, so a reference to one resolves as rejected rather than missing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CapabilityRegistrationRejections {
    diagnostics_by_family_and_identity:
        BTreeMap<&'static str, BTreeMap<String, Box<[CapabilityRegistrationDiagnostic]>>>,
}

impl CapabilityRegistrationRejections {
    pub(crate) fn from_validation(
        candidates: &[RegistrationCandidate],
        validation: &RegistrationValidationReport,
    ) -> Self {
        let mut diagnostics_by_family_and_identity = BTreeMap::new();
        for candidate in candidates {
            let (family, identity) = (candidate.family_name(), candidate.identity_text());
            if validation.accepts(family, identity) {
                continue;
            }
            let diagnostics = validation
                .diagnostics_of(family, identity)
                .cloned()
                .collect();
            diagnostics_by_family_and_identity
                .entry(family)
                .or_insert_with(BTreeMap::new)
                .insert(identity.to_owned(), diagnostics);
        }
        Self {
            diagnostics_by_family_and_identity,
        }
    }

    /// Refuses `identity` in `family` for `diagnostics`, as validation would.
    #[cfg(test)]
    pub(crate) fn with_refusal(
        mut self,
        family: &'static str,
        identity: &str,
        diagnostics: Vec<CapabilityRegistrationDiagnostic>,
    ) -> Self {
        self.diagnostics_by_family_and_identity
            .entry(family)
            .or_default()
            .insert(identity.to_owned(), diagnostics.into_boxed_slice());
        self
    }

    /// The diagnostics that refused `identity` in `family`, if validation
    /// refused it.
    pub(crate) fn diagnostics(
        &self,
        family: &'static str,
        identity: &str,
    ) -> Option<&[CapabilityRegistrationDiagnostic]> {
        self.diagnostics_by_family_and_identity
            .get(family)?
            .get(identity)
            .map(AsRef::as_ref)
    }
}

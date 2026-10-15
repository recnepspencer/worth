//! Immutable descriptive signing provenance. Decoding it never grants signing or approval authority.
mod binding;
#[cfg(test)]
mod tests;
pub use binding::ApplicationSigningProvenanceBinding;
use serde::{Deserialize, Serialize};

/// Coordinates, not a live graph reference or a principal permit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningEntityCoordinates {
    pub partition: u64,
    pub slot: u64,
    pub generation: u32,
}

/// Data-only transport. The native signed-creation owner supplies these parts;
/// caller-authored copies cannot satisfy its mandatory authentication handoff.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningProvenanceMetadata {
    pub program_revision: String,
    pub operation: String,
    pub binding: String,
    pub signer_issuer: String,
    pub signer_subject: String,
    pub purpose: String,
    pub verification_configuration: String,
    pub clock_source: String,
    pub clock_timeline: String,
}

impl ApplicationSigningProvenanceMetadata {
    /// Conservative JSON representation ceiling for the actual installed metadata.
    /// This is resource accounting, not a restriction on valid native clock names.
    pub fn maximum_encoded_bytes(&self) -> Option<usize> {
        [
            &self.program_revision,
            &self.operation,
            &self.binding,
            &self.signer_issuer,
            &self.signer_subject,
            &self.purpose,
            &self.verification_configuration,
            &self.clock_source,
            &self.clock_timeline,
        ]
        .into_iter()
        .try_fold(4096usize, |total, text| {
            total.checked_add(text.len().checked_mul(6)?)
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationSigningProvenanceParts {
    pub metadata: ApplicationSigningProvenanceMetadata,
    pub operation_meaning: [u8; 32],
    pub logical_request: [u8; 32],
    pub input: [u8; 32],
    pub subject: ApplicationSigningEntityCoordinates,
    pub signer: ApplicationSigningEntityCoordinates,
    pub signing_intent: [u8; 32],
    pub subject_coverage: [u8; 32],
    pub event_reference: [u8; 32],
    /// Named-clock coordinates. They are not UTC without a separately declared UTC timeline.
    pub issued_at_nanoseconds: u64,
    pub observed_at_nanoseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSigningProvenance(ApplicationSigningProvenanceParts);

impl ApplicationSigningProvenance {
    pub fn from_parts(
        parts: ApplicationSigningProvenanceParts,
    ) -> Result<Self, super::ApplicationValueValidationDenial> {
        let value = Self(parts);
        <ApplicationSigningProvenanceBinding as super::ApplicationScalarValueBinding>::validate(
            &value,
        )?;
        Ok(value)
    }
    pub fn parts(&self) -> &ApplicationSigningProvenanceParts {
        &self.0
    }
}

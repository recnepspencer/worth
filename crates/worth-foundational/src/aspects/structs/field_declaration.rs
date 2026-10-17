use crate::aspects::contracts::AbsenceLaw;
use crate::aspects::evolution::AspectEvolutionPolicy;
use crate::values::ScalarAspectType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FieldKey(String);

impl FieldKey {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.contains('.') || value.contains(char::is_whitespace) {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.0.capacity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldRequirement {
    Required,
    Optional,
    Defaulted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDeclaration {
    key: FieldKey,
    value_type: ScalarAspectType,
    requirement: FieldRequirement,
    absence: AbsenceLaw,
    evolution: AspectEvolutionPolicy,
    codec: Option<crate::CanonicalDigestId>,
}

impl FieldDeclaration {
    pub fn new(
        key: FieldKey,
        value_type: ScalarAspectType,
        requirement: FieldRequirement,
        absence: AbsenceLaw,
        evolution: AspectEvolutionPolicy,
    ) -> Option<Self> {
        if !requirement_matches_absence_law(requirement, absence) {
            return None;
        }

        Some(Self {
            key,
            value_type,
            requirement,
            absence,
            evolution,
            codec: None,
        })
    }

    pub fn key(&self) -> &FieldKey {
        &self.key
    }

    pub fn value_type(&self) -> ScalarAspectType {
        self.value_type
    }

    pub fn requirement(&self) -> FieldRequirement {
        self.requirement
    }

    pub fn absence(&self) -> AbsenceLaw {
        self.absence
    }

    pub fn evolution(&self) -> AspectEvolutionPolicy {
        self.evolution
    }

    /// Portable representation meaning, separate from its scalar carrier.
    /// This description grants neither decoding nor migration authority.
    pub fn with_codec(mut self, codec: crate::CanonicalDigestId) -> Self {
        self.codec = Some(codec);
        self
    }

    pub const fn codec(&self) -> Option<crate::CanonicalDigestId> {
        self.codec
    }
}

fn requirement_matches_absence_law(requirement: FieldRequirement, absence: AbsenceLaw) -> bool {
    matches!(
        (requirement, absence),
        (FieldRequirement::Required, AbsenceLaw::Required)
            | (FieldRequirement::Optional, AbsenceLaw::Optional)
            | (FieldRequirement::Defaulted, AbsenceLaw::Defaulted)
    )
}

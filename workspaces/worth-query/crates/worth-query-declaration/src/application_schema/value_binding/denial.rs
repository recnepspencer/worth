use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::ScalarAspectType;

/// Domain validation rejected a value at its entry-local binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationValueValidationDenial {
    binding_identity: WorthQueryPortableTypeIdentity,
    reason: &'static str,
}

impl ApplicationValueValidationDenial {
    pub fn rejected(
        binding_identity: WorthQueryPortableTypeIdentity,
        reason: &'static str,
    ) -> Self {
        Self {
            binding_identity,
            reason,
        }
    }

    pub const fn binding_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.binding_identity
    }

    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

/// Encoding failed before a value could enter Foundational scalar meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationValueEncodeDenial {
    Validation(ApplicationValueValidationDenial),
    CodecRejected {
        binding_identity: WorthQueryPortableTypeIdentity,
        reason: &'static str,
    },
}

impl ApplicationValueEncodeDenial {
    pub const fn binding_identity(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::Validation(denial) => denial.binding_identity(),
            Self::CodecRejected {
                binding_identity, ..
            } => binding_identity,
        }
    }
}

impl From<ApplicationValueValidationDenial> for ApplicationValueEncodeDenial {
    fn from(denial: ApplicationValueValidationDenial) -> Self {
        Self::Validation(denial)
    }
}

/// Decoding failed without manufacturing a domain value or permissive default.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationValueDecodeDenial {
    ScalarFamilyMismatch {
        binding_identity: WorthQueryPortableTypeIdentity,
        expected: ScalarAspectType,
        observed: ScalarAspectType,
    },
    CodecRejected {
        binding_identity: WorthQueryPortableTypeIdentity,
    },
    Validation(ApplicationValueValidationDenial),
}

impl ApplicationValueDecodeDenial {
    pub const fn binding_identity(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::ScalarFamilyMismatch {
                binding_identity, ..
            }
            | Self::CodecRejected { binding_identity } => binding_identity,
            Self::Validation(denial) => denial.binding_identity(),
        }
    }
}

impl From<ApplicationValueValidationDenial> for ApplicationValueDecodeDenial {
    fn from(denial: ApplicationValueValidationDenial) -> Self {
        Self::Validation(denial)
    }
}

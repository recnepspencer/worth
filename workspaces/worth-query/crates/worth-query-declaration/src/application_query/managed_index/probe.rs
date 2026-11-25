//! The probe's declared scalar and codec profile used for exact installation matching.
use crate::application_schema::{
    ApplicationCanonicalValueReference, ApplicationReadableScalarValueBinding,
    ApplicationValueDecodePosture,
};
use crate::archive_decode_authority::{ArchiveDecodeAuthority, ARCHIVE_DECODE};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{InlineBytes, ScalarAspectType};

#[cfg(test)]
mod tests;

/// Descriptive scalar and canonical codec profile transported with an index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableManagedIndexProbeParts {
    pub binding: WorthQueryPortableTypeIdentity,
    pub scalar_family: ScalarAspectType,
    pub unit: Option<String>,
    pub frame: Option<String>,
    pub canonical: Option<ApplicationCanonicalValueReference>,
    pub supports_admitted_decode: bool,
}

/// Complete portable probe meaning; installation still compares the actual typed binding.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationManagedIndexProbeMeaning {
    parts: WorthQueryPortableManagedIndexProbeParts,
}

impl ApplicationManagedIndexProbeMeaning {
    pub fn of<Binding: ApplicationReadableScalarValueBinding>() -> Self {
        let canonical = Binding::canonical_contract().map(|contract| contract.portable_reference());
        let supports_admitted_decode = Binding::SUPPORTS_ADMITTED_DECODE
            && Binding::Decode::erased_admitted_decode().is_some()
            && (canonical.is_some() != Binding::NATIVE_INPUT_ADMISSION.is_some());
        Self::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableManagedIndexProbeParts {
                binding: Binding::IDENTITY,
                scalar_family: Binding::SCALAR_FAMILY,
                unit: Binding::UNIT.map(|unit| unit.as_str().to_owned()),
                frame: Binding::FRAME.map(|frame| frame.as_str().to_owned()),
                canonical,
                supports_admitted_decode,
            },
        )
    }

    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableManagedIndexProbeParts,
    ) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &WorthQueryPortableManagedIndexProbeParts {
        &self.parts
    }

    pub fn has_supported_profile(&self) -> bool {
        self.parts.binding.is_valid()
            && self.parts.supports_admitted_decode
            && self.parts.canonical.as_ref().is_none_or(|canonical| {
                let parts = canonical.parts();
                if parts.shape.is_empty() || !parts.supports_admission {
                    return false;
                }
                match parts.protocol.as_str() {
                    "worth.canonical-json.v1" => {
                        self.parts.scalar_family == ScalarAspectType::String
                    }
                    "worth.domain-binary.v1" => {
                        self.parts.scalar_family == ScalarAspectType::InlineBytes
                            && parts.maximum_wire_bytes.is_some_and(|bytes| {
                                (1..=InlineBytes::MAXIMUM_LENGTH as u64).contains(&bytes)
                            })
                    }
                    _ => false,
                }
            })
    }
}

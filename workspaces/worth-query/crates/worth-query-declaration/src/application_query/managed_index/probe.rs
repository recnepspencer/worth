//! A canonical probe's full declared codec profile, never a decoder or admission.
use crate::application_schema::{
    ApplicationCanonicalValueReference, ApplicationReadableScalarValueBinding,
    ApplicationValueDecodePosture,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::ScalarAspectType;

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
        Self::from_untrusted_parts(WorthQueryPortableManagedIndexProbeParts {
            binding: Binding::IDENTITY,
            scalar_family: Binding::SCALAR_FAMILY,
            unit: Binding::UNIT.map(|unit| unit.as_str().to_owned()),
            frame: Binding::FRAME.map(|frame| frame.as_str().to_owned()),
            canonical: Binding::canonical_contract().map(|contract| contract.portable_reference()),
            supports_admitted_decode: Binding::SUPPORTS_ADMITTED_DECODE
                && Binding::Decode::erased_admitted_decode().is_some(),
        })
    }

    pub fn from_untrusted_parts(parts: WorthQueryPortableManagedIndexProbeParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &WorthQueryPortableManagedIndexProbeParts {
        &self.parts
    }

    pub fn has_supported_profile(&self) -> bool {
        self.parts.binding.is_valid()
            && self.parts.supports_admitted_decode
            && self.parts.canonical.as_ref().is_some_and(|canonical| {
                let parts = canonical.parts();
                parts.protocol == "worth.canonical-json.v1"
                    && !parts.shape.is_empty()
                    && parts.supports_admission
            })
    }
}

use crate::application_schema::{
    ApplicationCanonicalValueReference, ApplicationNativeInputDecodeProfile,
    ApplicationReadableScalarValueBinding, ApplicationValueDecodePosture,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::ScalarAspectType;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceValueBindingParts {
    pub identity: WorthQueryPortableTypeIdentity,
    pub scalar_family: ScalarAspectType,
    pub unit: Option<String>,
    pub frame: Option<String>,
    pub canonical: Option<ApplicationCanonicalValueReference>,
    pub supports_admitted_decode: bool,
    pub native_input_admission: Option<ApplicationNativeInputDecodeProfile>,
}

/// Complete portable codec description. Only comparison with the actual typed
/// installed binding can establish that a supported decoder exists.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceValueBindingReference {
    parts: ApplicationResourceValueBindingParts,
}

impl ApplicationResourceValueBindingReference {
    pub fn of<Binding: ApplicationReadableScalarValueBinding>() -> Self {
        Self {
            parts: ApplicationResourceValueBindingParts {
                identity: Binding::IDENTITY,
                scalar_family: Binding::SCALAR_FAMILY,
                unit: Binding::UNIT.map(|value| value.as_str().to_owned()),
                frame: Binding::FRAME.map(|value| value.as_str().to_owned()),
                canonical: Binding::canonical_contract()
                    .map(|contract| contract.portable_reference()),
                supports_admitted_decode: Binding::SUPPORTS_ADMITTED_DECODE
                    && Binding::Decode::erased_admitted_decode().is_some(),
                native_input_admission: Binding::NATIVE_INPUT_ADMISSION,
            },
        }
    }

    pub fn from_untrusted_parts(parts: ApplicationResourceValueBindingParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &ApplicationResourceValueBindingParts {
        &self.parts
    }

    pub fn matches_binding<Binding: ApplicationReadableScalarValueBinding>(&self) -> bool {
        self == &Self::of::<Binding>()
    }

    /// This checks declared compatibility, not the existence of a typed recipe.
    pub fn has_supported_profile(&self) -> bool {
        self.parts.identity.is_valid()
            && match &self.parts.canonical {
                Some(contract) => {
                    let contract = contract.parts();
                    contract.protocol == "worth.canonical-json.v1"
                        && !contract.shape.is_empty()
                        && contract.supports_admission
                        && self.parts.supports_admitted_decode
                }
                None => self.parts.native_input_admission.is_some(),
            }
    }
}

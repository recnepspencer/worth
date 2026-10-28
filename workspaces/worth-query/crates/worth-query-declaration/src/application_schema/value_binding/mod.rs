mod canonical;
// The canonical implementation is shared with pure owners. These are the
// complete seed vocabulary used by WorthValue's explicit audience-facade path.
pub use canonical::{
    decode_application_canonical_value, decode_application_canonical_value_with_admission,
    decode_application_document, decode_canonical_value, decode_canonical_value_with_admission,
    decode_delegated_canonical_value, decode_delegated_canonical_value_with_admission,
    encode_application_document, encode_canonical_value, encode_delegated_canonical_value,
    ApplicationCanonicalValueContract, CanonicalDocumentCodec, CanonicalValue, ValueCodecDenial,
    ValueDecodeAdmission, ValueDecodeBounds, ValueShape,
};
pub use worth_foundational::facade::{
    value_decode_error, ArrayDelegate, BoxDelegate, CanonicalValueSeed, DelegatedValueSeed,
    OptionalDelegate, PairDelegate, SequenceDelegate, UsizeDelegate, ValueDelegate,
    ValueFieldDefinition, ValuePrimitive, ValueSeedDelegate, ValueShapeBody, ValueShapeDefinition,
    ValueShapeEdge, ValueTaggedVariantDefinition, ValueVariantDefinition, WorthValue,
};
mod admitted;
#[cfg(test)]
mod admitted_tests;
#[cfg(test)]
mod canonical_tests;
mod denial;
mod encoded_scalar;
mod primitive;
mod recipe;
mod representation_identity;
mod scalar;
mod structured;

pub use denial::{
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, ApplicationValueValidationDenial,
};
pub use encoded_scalar::ApplicationEncodedScalarValue;
pub use primitive::{
    BoolApplicationValueBinding, I64ApplicationValueBinding, InternedStringApplicationValueBinding,
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
pub use recipe::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ErasedApplicationValueEncode,
    ErasedApplicationValueValidation,
};
pub use representation_identity::{ApplicationFrameIdentity, ApplicationUnitIdentity};
pub use scalar::{
    ApplicationIdentityScalarValueBinding, ApplicationNativeInputAdmissionDenial,
    ApplicationNativeInputDecodeProfile, ApplicationNativeInputDecodeResources,
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationSignedAggregateValueBinding, ApplicationValueDecodeAvailable,
    ApplicationValueDecodePosture, ApplicationValueDecodeUnavailable,
    ApplicationValueIdentityPosture, ApplicationValueIsIdentity, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateAvailable, ApplicationValueSignedAggregatePosture,
    ApplicationValueSignedAggregateUnavailable, ErasedApplicationSignedAggregateDecode,
    ErasedApplicationValueAdmittedDecode, ErasedApplicationValueDecode,
};
pub use structured::ApplicationStructuredValueBinding;

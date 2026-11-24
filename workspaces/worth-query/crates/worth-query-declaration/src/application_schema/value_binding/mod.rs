mod canonical;
// The canonical implementation is shared with pure owners. These are the
// complete seed vocabulary used by WorthValue's explicit audience-facade path.
pub use canonical::{
    decode_application_binary_value, decode_application_canonical_value,
    decode_application_canonical_value_with_admission, decode_application_document,
    decode_canonical_value, decode_canonical_value_with_admission,
    decode_delegated_canonical_value, decode_delegated_canonical_value_with_admission,
    encode_application_binary_value, encode_application_document, encode_canonical_value,
    encode_delegated_canonical_value, encode_stored_canonical_value,
    ApplicationCanonicalValueContract, ApplicationCanonicalValueReference,
    ApplicationCanonicalValueReferenceParts, BinaryValueDelegate, BinaryValueWriter,
    CanonicalDocumentCodec, CanonicalValue, ValueCodecDenial, ValueDecodeAdmission,
    ValueDecodeBounds, ValueShape,
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
mod entity_ref;
mod native_scalar;
mod persistent_name;
mod primitive;
mod recipe;
mod representation_identity;
mod scalar;
mod structured;

pub use denial::{
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, ApplicationValueValidationDenial,
};
pub use encoded_scalar::ApplicationEncodedScalarValue;
pub use entity_ref::EntityRefApplicationValueBinding;
pub use native_scalar::{
    CanonicalDateApplicationValueBinding, CanonicalF32ApplicationValueBinding,
    CanonicalF64ApplicationValueBinding, CanonicalTimeApplicationValueBinding,
    CanonicalTimestampApplicationValueBinding, CanonicalTimestampTzApplicationValueBinding,
    InlineBytesApplicationValueBinding, UuidApplicationValueBinding,
};
pub use persistent_name::{
    NamedTargetApplicationValueBinding, WorthQueryNameApplicationValueBinding,
};
pub use primitive::{
    BoolApplicationValueBinding, ContentRefApplicationValueBinding, I16ApplicationValueBinding,
    I32ApplicationValueBinding, I64ApplicationValueBinding, I8ApplicationValueBinding,
    InternedStringApplicationValueBinding, StringApplicationValueBinding,
    U16ApplicationValueBinding, U32ApplicationValueBinding, U64ApplicationValueBinding,
    U8ApplicationValueBinding,
};
pub use recipe::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingOwnerKind, ApplicationFieldBindingRecipe,
    ErasedApplicationValueEncode, ErasedApplicationValueValidation,
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

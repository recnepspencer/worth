use std::any::{Any, TypeId};

use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

use super::{
    ApplicationFrameIdentity, ApplicationScalarValueBinding, ApplicationUnitIdentity,
    ApplicationValueDecodePosture, ApplicationValueEncodeDenial, ApplicationValueIdentityPosture,
    ApplicationValueSignedAggregatePosture, ApplicationValueValidationDenial,
    ErasedApplicationSignedAggregateDecode, ErasedApplicationValueAdmittedDecode,
    ErasedApplicationValueDecode,
};
use crate::application_schema::ApplicationSchemaMember;

pub type ErasedApplicationValueValidation =
    fn(&dyn Any) -> Result<(), ApplicationValueValidationDenial>;
pub type ErasedApplicationValueEncode =
    fn(&dyn Any) -> Result<AspectValue, ApplicationValueEncodeDenial>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationFieldBindingLocus {
    entity: String,
    aspect: String,
    field: String,
}

impl ApplicationFieldBindingLocus {
    pub fn new(entity: &str, aspect: &str, field: &str) -> Self {
        Self {
            entity: entity.to_owned(),
            aspect: aspect.to_owned(),
            field: field.to_owned(),
        }
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationFieldBindingRecipe {
    locus: ApplicationFieldBindingLocus,
    binding_identity: WorthQueryPortableTypeIdentity,
    scalar_family: ScalarAspectType,
    unit: Option<ApplicationUnitIdentity>,
    frame: Option<ApplicationFrameIdentity>,
    binding_type: TypeId,
    value_type: TypeId,
    validate: ErasedApplicationValueValidation,
    encode: ErasedApplicationValueEncode,
    decode: Option<ErasedApplicationValueDecode>,
    admitted_decode: Option<ErasedApplicationValueAdmittedDecode>,
    supports_admitted_decode: bool,
    identity_capable: bool,
    signed_aggregate_decode: Option<ErasedApplicationSignedAggregateDecode>,
    canonical: Option<super::ApplicationCanonicalValueContract>,
}

impl ApplicationFieldBindingRecipe {
    pub(crate) fn of<Binding: ApplicationScalarValueBinding>(
        locus: ApplicationFieldBindingLocus,
    ) -> Self {
        let canonical = Binding::canonical_contract();
        Self {
            locus,
            binding_identity: Binding::IDENTITY,
            scalar_family: Binding::SCALAR_FAMILY,
            unit: Binding::UNIT,
            frame: Binding::FRAME,
            binding_type: TypeId::of::<Binding>(),
            value_type: TypeId::of::<Binding::Value>(),
            validate: erased_validate::<Binding>,
            encode: erased_encode::<Binding>,
            decode: Binding::Decode::erased_decode(),
            admitted_decode: Binding::Decode::erased_admitted_decode(),
            supports_admitted_decode: Binding::SUPPORTS_ADMITTED_DECODE
                && canonical
                    .as_ref()
                    .is_none_or(|contract| contract.supports_admission()),
            identity_capable: Binding::Identity::CAPABLE,
            signed_aggregate_decode: Binding::SignedAggregate::erased_decode(),
            canonical,
        }
    }

    pub fn locus(&self) -> &ApplicationFieldBindingLocus {
        &self.locus
    }

    pub fn binding_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.binding_identity
    }

    pub const fn canonical_contract(&self) -> Option<&super::ApplicationCanonicalValueContract> {
        self.canonical.as_ref()
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn unit(&self) -> Option<ApplicationUnitIdentity> {
        self.unit
    }

    pub const fn frame(&self) -> Option<ApplicationFrameIdentity> {
        self.frame
    }

    pub fn binding_type(&self) -> TypeId {
        self.binding_type
    }

    pub fn value_type(&self) -> TypeId {
        self.value_type
    }

    pub const fn validate(&self) -> ErasedApplicationValueValidation {
        self.validate
    }

    pub const fn encode(&self) -> ErasedApplicationValueEncode {
        self.encode
    }

    pub const fn decode(&self) -> Option<ErasedApplicationValueDecode> {
        self.decode
    }

    pub const fn admitted_decode(&self) -> Option<ErasedApplicationValueAdmittedDecode> {
        self.admitted_decode
    }

    pub const fn supports_admitted_decode(&self) -> bool {
        self.supports_admitted_decode && self.admitted_decode.is_some()
    }

    pub const fn identity_capable(&self) -> bool {
        self.identity_capable
    }
    pub const fn signed_aggregate_decode(&self) -> Option<ErasedApplicationSignedAggregateDecode> {
        self.signed_aggregate_decode
    }

    pub(crate) fn has_same_contract(&self, other: &Self) -> bool {
        self.binding_identity == other.binding_identity
            && self.canonical == other.canonical
            && self.scalar_family == other.scalar_family
            && self.unit == other.unit
            && self.frame == other.frame
            && self.binding_type == other.binding_type
            && self.value_type == other.value_type
            && self.decode.is_some() == other.decode.is_some()
            && self.supports_admitted_decode == other.supports_admitted_decode
            && self.admitted_decode.is_some() == other.admitted_decode.is_some()
            && self.identity_capable == other.identity_capable
            && self.signed_aggregate_decode.is_some() == other.signed_aggregate_decode.is_some()
    }

    pub(crate) fn matches_member(&self, member: &ApplicationSchemaMember) -> bool {
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            scalar_family,
            value_type,
            unit,
            frame,
            canonical,
            ..
        } = member
        else {
            return false;
        };
        self.locus.entity() == entity
            && self.locus.aspect() == aspect
            && self.locus.field() == field
            && self.scalar_family == *scalar_family
            && self.binding_identity.as_str() == value_type
            && self.unit.map(ApplicationUnitIdentity::as_str) == unit.as_deref()
            && self.frame.map(ApplicationFrameIdentity::as_str) == frame.as_deref()
            && self.canonical.as_ref() == canonical.as_ref()
    }
}

impl PartialEq for ApplicationFieldBindingRecipe {
    fn eq(&self, other: &Self) -> bool {
        self.locus == other.locus && self.has_same_contract(other)
    }
}

impl Eq for ApplicationFieldBindingRecipe {}

fn erased_validate<Binding: ApplicationScalarValueBinding>(
    value: &dyn Any,
) -> Result<(), ApplicationValueValidationDenial> {
    let value = value.downcast_ref::<Binding::Value>().ok_or_else(|| {
        ApplicationValueValidationDenial::rejected(Binding::IDENTITY, "binding value type mismatch")
    })?;
    Binding::validate(value)
}

fn erased_encode<Binding: ApplicationScalarValueBinding>(
    value: &dyn Any,
) -> Result<AspectValue, ApplicationValueEncodeDenial> {
    let value = value.downcast_ref::<Binding::Value>().ok_or_else(|| {
        ApplicationValueValidationDenial::rejected(Binding::IDENTITY, "binding value type mismatch")
    })?;
    Binding::validate(value)?;
    Binding::encode(value)
}

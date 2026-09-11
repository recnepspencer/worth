use crate::portable_identity::WorthQueryPortableTypeIdentity;
use std::any::Any;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

use super::{
    ApplicationFrameIdentity, ApplicationUnitIdentity, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueValidationDenial,
};

/// Entry-owned exact scalar representation for a Query-free domain value.
pub trait ApplicationScalarValueBinding: 'static {
    type Value: 'static;
    type Unit: 'static;
    type Decode: ApplicationValueDecodePosture<Self>;
    type Identity: ApplicationValueIdentityPosture;
    type SignedAggregate: ApplicationValueSignedAggregatePosture<Self>;

    const IDENTITY_NAME: &'static str;
    const IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(Self::IDENTITY_NAME);
    const SCALAR_FAMILY: ScalarAspectType;
    const UNIT: Option<ApplicationUnitIdentity> = None;
    const FRAME: Option<ApplicationFrameIdentity> = None;

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial>;

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial>;
}

/// Scalar binding that can reconstruct its exact domain value.
pub trait ApplicationReadableScalarValueBinding: ApplicationScalarValueBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial>;
}

/// Readable binding suitable for stable application identity fields.
pub trait ApplicationIdentityScalarValueBinding: ApplicationReadableScalarValueBinding {}

/// Readable binding suitable for checked provider-derived signed sums.
pub trait ApplicationSignedAggregateValueBinding: ApplicationReadableScalarValueBinding {
    fn decode_aggregate(value: i64) -> Result<Self::Value, ApplicationValueDecodeDenial>;
}

pub type ErasedApplicationValueDecode =
    fn(&AspectValue) -> Result<Box<dyn Any>, ApplicationValueDecodeDenial>;
pub type ErasedApplicationSignedAggregateDecode =
    fn(i64) -> Result<Box<dyn Any>, ApplicationValueDecodeDenial>;

pub trait ApplicationValueDecodePosture<Binding: ApplicationScalarValueBinding + ?Sized> {
    fn erased_decode() -> Option<ErasedApplicationValueDecode>;
}

pub trait ApplicationValueIdentityPosture {
    const CAPABLE: bool;
}

pub trait ApplicationValueSignedAggregatePosture<Binding: ApplicationScalarValueBinding + ?Sized> {
    fn erased_decode() -> Option<ErasedApplicationSignedAggregateDecode>;
}

pub struct ApplicationValueDecodeUnavailable;
pub struct ApplicationValueDecodeAvailable;
pub struct ApplicationValueIsNotIdentity;
pub struct ApplicationValueIsIdentity;
pub struct ApplicationValueSignedAggregateUnavailable;
pub struct ApplicationValueSignedAggregateAvailable;

impl<Binding: ApplicationScalarValueBinding> ApplicationValueDecodePosture<Binding>
    for ApplicationValueDecodeUnavailable
{
    fn erased_decode() -> Option<ErasedApplicationValueDecode> {
        None
    }
}

impl<Binding> ApplicationValueDecodePosture<Binding> for ApplicationValueDecodeAvailable
where
    Binding: ApplicationReadableScalarValueBinding,
{
    fn erased_decode() -> Option<ErasedApplicationValueDecode> {
        Some(|value| {
            let value = Binding::decode(value)?;
            Binding::validate(&value)?;
            Ok(Box::new(value) as Box<dyn Any>)
        })
    }
}

impl ApplicationValueIdentityPosture for ApplicationValueIsNotIdentity {
    const CAPABLE: bool = false;
}

impl ApplicationValueIdentityPosture for ApplicationValueIsIdentity {
    const CAPABLE: bool = true;
}

impl<Binding: ApplicationScalarValueBinding> ApplicationValueSignedAggregatePosture<Binding>
    for ApplicationValueSignedAggregateUnavailable
{
    fn erased_decode() -> Option<ErasedApplicationSignedAggregateDecode> {
        None
    }
}

impl<Binding> ApplicationValueSignedAggregatePosture<Binding>
    for ApplicationValueSignedAggregateAvailable
where
    Binding: ApplicationSignedAggregateValueBinding,
{
    fn erased_decode() -> Option<ErasedApplicationSignedAggregateDecode> {
        Some(|value| {
            let value = Binding::decode_aggregate(value)?;
            Binding::validate(&value)?;
            Ok(Box::new(value) as Box<dyn Any>)
        })
    }
}

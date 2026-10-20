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
    /// Whether the owner supplies resource-admitted canonical decoding.
    const SUPPORTS_ADMITTED_DECODE: bool = false;

    /// Derived complete wire meaning, when this binding adopts canonical values.
    fn canonical_contract() -> Option<super::ApplicationCanonicalValueContract> {
        None
    }
    /// Explicit admission for using this decoder in a native operation body.
    /// Native code computes this data-only bound before calling the decoder.
    /// It must cover decoding, validation, boxing and temporary allocation.
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> = None;

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial>;

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationNativeInputDecodeResources {
    work_units: u64,
    peak_memory_bytes: u64,
}

/// Conservative cost of decoding a native scalar, expressed without a callback.
/// Each per-byte term applies to the scalar's owned allocation capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationNativeInputDecodeProfile {
    work_base: u64,
    work_per_byte: u64,
    peak_memory_base: u64,
    peak_memory_per_byte: u64,
}

impl ApplicationNativeInputDecodeProfile {
    pub const fn new(
        work_base: u64,
        work_per_byte: u64,
        peak_memory_base: u64,
        peak_memory_per_byte: u64,
    ) -> Self {
        Self {
            work_base,
            work_per_byte,
            peak_memory_base,
            peak_memory_per_byte,
        }
    }

    pub fn requirements(self, bytes: u64) -> Option<ApplicationNativeInputDecodeResources> {
        Some(ApplicationNativeInputDecodeResources::new(
            self.work_per_byte
                .checked_mul(bytes)?
                .checked_add(self.work_base)?,
            self.peak_memory_per_byte
                .checked_mul(bytes)?
                .checked_add(self.peak_memory_base)?,
        ))
    }
}

impl ApplicationNativeInputDecodeResources {
    pub const fn new(work_units: u64, peak_memory_bytes: u64) -> Self {
        Self {
            work_units,
            peak_memory_bytes,
        }
    }

    pub const fn work_units(self) -> u64 {
        self.work_units
    }
    pub const fn peak_memory_bytes(self) -> u64 {
        self.peak_memory_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationNativeInputAdmissionDenial {
    InvalidValue,
    CapacityOverflow,
}

/// Scalar binding that can reconstruct its exact domain value.
pub trait ApplicationReadableScalarValueBinding: ApplicationScalarValueBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial>;

    /// Native request dispatch. Canonical owners must opt in explicitly; a
    /// representation-only decoder is never an admitted decoder fallback.
    fn decode_admitted(
        value: &AspectValue,
        _admission: &mut super::ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        if Self::canonical_contract().is_some() {
            return Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
            });
        }
        Self::decode(value)
    }
}

/// Readable binding suitable for stable application identity fields.
pub trait ApplicationIdentityScalarValueBinding: ApplicationReadableScalarValueBinding {}

/// Readable binding suitable for checked provider-derived signed sums.
pub trait ApplicationSignedAggregateValueBinding: ApplicationReadableScalarValueBinding {
    fn decode_aggregate(value: i64) -> Result<Self::Value, ApplicationValueDecodeDenial>;
}

pub type ErasedApplicationValueDecode =
    fn(&AspectValue) -> Result<Box<dyn Any>, ApplicationValueDecodeDenial>;
pub type ErasedApplicationValueAdmittedDecode =
    fn(
        &AspectValue,
        &mut super::ValueDecodeAdmission<'_>,
    ) -> Result<Box<dyn Any>, ApplicationValueDecodeDenial>;
pub type ErasedApplicationSignedAggregateDecode =
    fn(i64) -> Result<Box<dyn Any>, ApplicationValueDecodeDenial>;

pub trait ApplicationValueDecodePosture<Binding: ApplicationScalarValueBinding + ?Sized> {
    fn erased_decode() -> Option<ErasedApplicationValueDecode>;
    fn erased_admitted_decode() -> Option<ErasedApplicationValueAdmittedDecode> {
        None
    }
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
    fn erased_admitted_decode() -> Option<ErasedApplicationValueAdmittedDecode> {
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

    fn erased_admitted_decode() -> Option<ErasedApplicationValueAdmittedDecode> {
        Some(super::admitted::decode_erased::<Binding>)
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

#[cfg(test)]
mod native_input_admission_tests {
    use super::ApplicationNativeInputDecodeProfile;

    #[test]
    fn static_decoder_profile_bounds_and_overflow() {
        let profile = ApplicationNativeInputDecodeProfile::new(3, 2, 8, 4);
        let quoted = profile.requirements(5).unwrap();
        assert_eq!(quoted.work_units(), 13);
        assert_eq!(quoted.peak_memory_bytes(), 28);
        assert!(profile.requirements(u64::MAX).is_none());
    }
}

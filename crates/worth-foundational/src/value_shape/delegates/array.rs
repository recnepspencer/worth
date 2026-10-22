//! Fixed cardinality uses the existing tuple grammar and drops partial values normally.
use super::*;
use serde::de::{Error, SeqAccess, Visitor};

pub struct ArrayDelegate<Child, const N: usize>(PhantomData<fn() -> Child>);
impl<Child: ValueSeedDelegate, const N: usize> ValueSeedDelegate for ArrayDelegate<Child, N>
where
    [Child::Value; N]: Serialize,
{
    type Value = [Child::Value; N];
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = ValueShapeBody::Tuple(&[Child::EDGE; N]);
    fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D> {
        let child = bounds.descend()?;
        for value in value {
            Child::validate(value, child)?;
        }
        Ok(())
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        mut admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Self::Value, Decoder::Error> {
        struct Array<'a, 'r, C, const N: usize>(
            ValueDecodeBounds,
            Option<&'a mut ValueDecodeAdmission<'r>>,
            PhantomData<fn() -> C>,
        );
        impl<'de, C: ValueSeedDelegate, const N: usize> Visitor<'de> for Array<'_, '_, C, N> {
            type Value = [C::Value; N];
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "exactly {N} canonical array elements")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                mut self,
                mut access: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values: [Option<C::Value>; N] = std::array::from_fn(|_| None);
                for slot in &mut values {
                    *slot = Some(
                        access
                            .next_element_seed(DelegatedValueSeed::<C>::new(
                                self.0,
                                self.1.as_deref_mut(),
                            ))?
                            .ok_or_else(|| A::Error::custom(D::Malformed))?,
                    );
                }
                access.next_element_seed(super::super::containers::RejectAdditionalElement(
                    self.1.as_deref_mut(),
                ))?;
                Ok(values.map(|value| value.expect("every fixed array element was decoded")))
            }
        }
        let child = bounds.descend().map_err(|cause| {
            super::super::value_decode_error::<Decoder::Error>(cause, admission.as_deref_mut())
        })?;
        decoder.deserialize_tuple(N, Array::<Child, N>(child, admission, PhantomData))
    }
}

impl<Value: CanonicalValue, const N: usize> CanonicalValue for [Value; N]
where
    [Value; N]: Serialize,
{
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = ArrayDelegate::<ValueDelegate<Value>, N>::SHAPE_BODY;
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), D> {
        ArrayDelegate::<ValueDelegate<Value>, N>::validate(self, bounds)
    }
    fn deserialize_bounded<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, Decoder::Error> {
        ArrayDelegate::<ValueDelegate<Value>, N>::deserialize(decoder, bounds, None)
    }
    fn deserialize_admitted<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        ArrayDelegate::<ValueDelegate<Value>, N>::deserialize(decoder, bounds, Some(admission))
    }
    fn deserialize_reporting<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        reporter: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, Decoder::Error> {
        ArrayDelegate::<ValueDelegate<Value>, N>::deserialize(decoder, bounds, Some(reporter))
    }
}

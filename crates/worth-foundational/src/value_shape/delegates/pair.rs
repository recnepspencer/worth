use super::*;
use serde::de::{Error, SeqAccess, Visitor};

pub struct PairDelegate<Left, Right>(PhantomData<fn() -> (Left, Right)>);
impl<Left: ValueSeedDelegate, Right: ValueSeedDelegate> ValueSeedDelegate
    for PairDelegate<Left, Right>
{
    type Value = (Left::Value, Right::Value);
    const SUPPORTS_ADMISSION: bool = true;
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { ValueShapeBody::Tuple(&[Left::EDGE, Right::EDGE]) };
    fn validate(value: &Self::Value, bounds: ValueDecodeBounds) -> Result<(), D> {
        let child = bounds.descend()?;
        Left::validate(&value.0, child)?;
        Right::validate(&value.1, child)
    }
    fn deserialize<'de, Decoder: serde::Deserializer<'de>>(
        decoder: Decoder,
        bounds: ValueDecodeBounds,
        admission: Option<&mut ValueDecodeAdmission<'_>>,
    ) -> Result<Self::Value, Decoder::Error> {
        struct Pair<'a, 'r, L, R>(
            ValueDecodeBounds,
            Option<&'a mut ValueDecodeAdmission<'r>>,
            PhantomData<fn() -> (L, R)>,
        );
        impl<'de, L: ValueSeedDelegate, R: ValueSeedDelegate> Visitor<'de> for Pair<'_, '_, L, R> {
            type Value = (L::Value, R::Value);
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an exact bounded pair")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                mut self,
                mut access: A,
            ) -> Result<Self::Value, A::Error> {
                let left = access
                    .next_element_seed(DelegatedValueSeed::<L>::new(self.0, self.1.as_deref_mut()))?
                    .ok_or_else(|| A::Error::custom(D::Malformed))?;
                let right = access
                    .next_element_seed(DelegatedValueSeed::<R>::new(self.0, self.1.as_deref_mut()))?
                    .ok_or_else(|| A::Error::custom(D::Malformed))?;
                access.next_element_seed(super::super::containers::RejectAdditionalElement)?;
                Ok((left, right))
            }
        }
        decoder.deserialize_tuple(
            2,
            Pair::<Left, Right>(
                bounds.descend().map_err(Decoder::Error::custom)?,
                admission,
                PhantomData,
            ),
        )
    }
}

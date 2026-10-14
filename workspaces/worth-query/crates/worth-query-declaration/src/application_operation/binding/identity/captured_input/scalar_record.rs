//! Read the existing canonical serde grammar; no JSON or second serialization.
use super::super::encoder::tag;
use super::{ApplicationCanonicalInputDenial as Denial, ApplicationCanonicalInputScalar};
use worth_foundational::facade::{AspectValue, CanonicalF32, CanonicalF64, ScalarAspectType};

pub(super) fn select<'a>(
    bytes: &'a [u8],
    member: &str,
    family: ScalarAspectType,
) -> Result<ApplicationCanonicalInputScalar<'a>, Denial> {
    let mut input = Cursor(bytes);
    if input.byte()? != tag::STRUCT {
        return Err(Denial::UnsupportedShape);
    }
    input.text()?; // Exact serialized struct name remains in the captured identity.
    let mut selected = None;
    loop {
        match input.byte()? {
            0 => break,
            1 => {}
            _ => return Err(Denial::UnsupportedShape),
        }
        let name = input.text()?;
        let start = input.0;
        input.skip_scalar()?;
        if name == member {
            if selected.is_some() {
                return Err(Denial::DuplicateMember);
            }
            selected = Some(&start[..start.len() - input.0.len()]);
        }
    }
    if !input.0.is_empty() {
        return Err(Denial::UnsupportedShape);
    }
    Cursor(selected.ok_or(Denial::MissingMember)?).scalar(family)
}

struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], Denial> {
        let value = self.0.get(..length).ok_or(Denial::UnsupportedShape)?;
        self.0 = &self.0[length..];
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, Denial> {
        Ok(self.take(1)?[0])
    }
    fn integer(&mut self) -> Result<u128, Denial> {
        let mut value = 0_u128;
        for shift in (0..=126).step_by(7) {
            let byte = self.byte()?;
            let part = u128::from(byte & 127)
                .checked_shl(shift)
                .ok_or(Denial::InvalidScalar)?;
            if shift == 126 && byte & 127 > 3 {
                return Err(Denial::InvalidScalar);
            }
            value |= part;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(Denial::InvalidScalar)
    }
    fn text(&mut self) -> Result<&'a str, Denial> {
        let length = usize::try_from(self.integer()?).map_err(|_| Denial::InvalidScalar)?;
        std::str::from_utf8(self.take(length)?).map_err(|_| Denial::InvalidScalar)
    }
    fn skip_scalar(&mut self) -> Result<(), Denial> {
        match self.byte()? {
            tag::BOOL => {
                self.take(1)?;
            }
            tag::SIGNED | tag::UNSIGNED => {
                self.integer()?;
            }
            tag::F32 | tag::CHAR => {
                self.take(4)?;
            }
            tag::F64 => {
                self.take(8)?;
            }
            tag::STR | tag::BYTES => {
                let length = usize::try_from(self.integer()?).map_err(|_| Denial::InvalidScalar)?;
                self.take(length)?;
            }
            tag::NONE | tag::UNIT => {}
            _ => return Err(Denial::UnsupportedShape),
        }
        Ok(())
    }
    fn scalar(
        mut self,
        family: ScalarAspectType,
    ) -> Result<ApplicationCanonicalInputScalar<'a>, Denial> {
        use ScalarAspectType as F;
        let value = match (self.byte()?, family) {
            (tag::BOOL, F::Bool) => match self.byte()? {
                0 => AspectValue::Bool(false),
                1 => AspectValue::Bool(true),
                _ => return Err(Denial::InvalidScalar),
            },
            (tag::SIGNED, F::Int8 | F::Int16 | F::Int32 | F::Int64) => {
                let bits = self.integer()?;
                let value = ((bits >> 1) as i128) ^ -((bits & 1) as i128);
                match family {
                    F::Int8 => {
                        AspectValue::Int8(value.try_into().map_err(|_| Denial::InvalidScalar)?)
                    }
                    F::Int16 => {
                        AspectValue::Int16(value.try_into().map_err(|_| Denial::InvalidScalar)?)
                    }
                    F::Int32 => {
                        AspectValue::Int32(value.try_into().map_err(|_| Denial::InvalidScalar)?)
                    }
                    _ => AspectValue::Int64(value.try_into().map_err(|_| Denial::InvalidScalar)?),
                }
            }
            (tag::UNSIGNED, F::UInt8 | F::UInt16 | F::UInt32 | F::UInt64) => {
                let value = self.integer()?;
                match family {
                    F::UInt8 => {
                        AspectValue::UInt8(value.try_into().map_err(|_| Denial::InvalidScalar)?)
                    }
                    F::UInt16 => {
                        AspectValue::UInt16(value.try_into().map_err(|_| Denial::InvalidScalar)?)
                    }
                    F::UInt32 => {
                        AspectValue::UInt32(value.try_into().map_err(|_| Denial::InvalidScalar)?)
                    }
                    _ => AspectValue::UInt64(value.try_into().map_err(|_| Denial::InvalidScalar)?),
                }
            }
            (tag::F32, F::Float32) => {
                let bits = u32::from_be_bytes(self.take(4)?.try_into().unwrap());
                let value = CanonicalF32::from_bits(bits);
                if value.bits() != bits {
                    return Err(Denial::InvalidScalar);
                }
                AspectValue::Float32(value)
            }
            (tag::F64, F::Float64) => {
                let bits = u64::from_be_bytes(self.take(8)?.try_into().unwrap());
                let value = CanonicalF64::from_bits(bits);
                if value.bits() != bits {
                    return Err(Denial::InvalidScalar);
                }
                AspectValue::Float64(value)
            }
            (tag::STR, F::String) => {
                return Ok(ApplicationCanonicalInputScalar::Text(self.text()?))
            }
            _ => return Err(Denial::InvalidScalar),
        };
        Ok(ApplicationCanonicalInputScalar::Inline(value))
    }
}

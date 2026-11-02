//! Allocation-free preflight for the concrete canonical scalar body.
use super::super::CanonicalDecodeStorage;
use super::tags;

pub(crate) fn body_decoded_bytes<E>(
    length: usize,
    next: impl FnMut() -> Result<u8, E>,
    malformed: impl Fn() -> E,
) -> Result<CanonicalDecodeStorage, E> {
    let mut input = Input {
        remaining: length,
        next,
        malformed,
    };
    let bytes = match input.byte()? {
        tags::INLINE_BYTES => {
            let length = input.length()?;
            worth_foundational::InlineBytes::admit_length(length)
                .map_err(|_| (input.malformed)())?;
            input.skip(length)?;
            length
        }
        tags::NULL => 0,
        tags::BOOL | tags::INT8 | tags::UINT8 => {
            input.skip(1)?;
            0
        }
        tags::INT16 | tags::UINT16 => {
            input.skip(2)?;
            0
        }
        tags::INT32 | tags::UINT32 | tags::FLOAT32 | tags::DATE => {
            input.skip(4)?;
            0
        }
        tags::INT64
        | tags::UINT64
        | tags::FLOAT64
        | tags::BYTES
        | tags::TIME
        | tags::TIMESTAMP
        | tags::CONTENT_REF => {
            input.skip(8)?;
            0
        }
        tags::UUID | tags::ENTITY_REF => {
            input.skip(16)?;
            0
        }
        tags::TIMESTAMP_TZ => {
            input.skip(12)?;
            0
        }
        tags::STRING => match input.byte()? {
            tags::RAW_STRING => input.string()?,
            tags::SYMBOL_STRING => {
                input.skip(4)?;
                0
            }
            _ => return Err((input.malformed)()),
        },
        tags::DECIMAL | tags::BIG_INT => input.string()?,
        tags::RATIONAL => {
            let numerator = input.string()?;
            numerator
                .checked_add(input.string()?)
                .ok_or_else(&input.malformed)?
        }
        _ => return Err((input.malformed)()),
    };
    if input.remaining != 0 {
        return Err((input.malformed)());
    }
    Ok(CanonicalDecodeStorage {
        retained_bytes: u64::try_from(bytes).map_err(|_| (input.malformed)())?,
        transient_bytes: 0,
    })
}

pub(crate) fn framed_decoded_bytes<E>(
    length: usize,
    mut next: impl FnMut() -> Result<u8, E>,
    malformed: impl Fn() -> E,
) -> Result<CanonicalDecodeStorage, E> {
    let remaining = length.checked_sub(4).ok_or_else(&malformed)?;
    let body = u32::from_le_bytes([next()?, next()?, next()?, next()?]) as usize;
    if body != remaining {
        return Err(malformed());
    }
    body_decoded_bytes(body, next, malformed)
}
struct Input<F, M> {
    remaining: usize,
    next: F,
    malformed: M,
}
impl<E, F: FnMut() -> Result<u8, E>, M: Fn() -> E> Input<F, M> {
    fn byte(&mut self) -> Result<u8, E> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(&self.malformed)?;
        (self.next)()
    }
    fn skip(&mut self, length: usize) -> Result<(), E> {
        if length > self.remaining {
            return Err((self.malformed)());
        }
        for _ in 0..length {
            self.byte()?;
        }
        Ok(())
    }
    fn string(&mut self) -> Result<usize, E> {
        let length = self.length()?;
        self.skip(length)?;
        // Canonical numeric constructors only move their validated String;
        // raw text similarly owns exactly this decoded spelling.
        Ok(length)
    }
    fn length(&mut self) -> Result<usize, E> {
        Ok(u32::from_le_bytes([self.byte()?, self.byte()?, self.byte()?, self.byte()?]) as usize)
    }
}

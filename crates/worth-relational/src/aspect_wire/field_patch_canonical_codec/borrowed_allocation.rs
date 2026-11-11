//! Concrete field-patch rosters and nested binary owners, before decode.
use super::super::{locator_decoded_bytes, value_body_decoded_bytes, CanonicalDecodeStorage};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

pub(crate) fn decoded_bytes<E>(
    length: usize,
    next: impl FnMut() -> Result<u8, E>,
    malformed: impl Fn() -> E + Copy,
) -> Result<CanonicalDecodeStorage, E> {
    let mut input = Input {
        remaining: length,
        next,
        malformed,
    };
    let count = input.u32()?;
    if count > input.remaining / 8 {
        return Err(malformed());
    }
    // Keep the actual boxed output separate from additional sorting/copying
    // storage, so a temporary roster charge does not escape with the value.
    let retained = std::alloc::Layout::array::<(AspectFieldLocator, AspectValue)>(count)
        .map_err(|_| malformed())?
        .size();
    let triples = std::alloc::Layout::array::<(AspectFieldLocator, AspectValue, usize)>(count)
        .map_err(|_| malformed())?
        .size();
    let transient = triples.checked_add(retained).ok_or_else(malformed)?;
    let mut bytes = CanonicalDecodeStorage {
        retained_bytes: u64::try_from(retained).map_err(|_| malformed())?,
        transient_bytes: u64::try_from(transient).map_err(|_| malformed())?,
    };
    let mut largest_locator = 0;
    for _ in 0..count {
        let locator_length = input.u32()?;
        if locator_length > input.remaining {
            return Err(malformed());
        }
        largest_locator = largest_locator.max(locator_length);
        let locator = locator_decoded_bytes(locator_length, || input.byte(), malformed)?;
        bytes.retained_bytes = bytes
            .retained_bytes
            .checked_add(locator.retained_bytes)
            .ok_or_else(malformed)?;
        bytes.transient_bytes = bytes
            .transient_bytes
            .checked_add(locator.transient_bytes)
            .ok_or_else(malformed)?;
        let value_length = input.u32()?;
        if value_length > input.remaining {
            return Err(malformed());
        }
        let value = value_body_decoded_bytes(value_length, || input.byte(), malformed)?;
        bytes.retained_bytes = bytes
            .retained_bytes
            .checked_add(value.retained_bytes)
            .ok_or_else(malformed)?;
        bytes.transient_bytes = bytes
            .transient_bytes
            .checked_add(value.transient_bytes)
            .ok_or_else(malformed)?;
    }
    if input.remaining != 0 {
        return Err(malformed());
    }
    // Canonical re-encoding briefly owns one locator buffer while appending
    // it to the outer patch byte buffer (admitted separately by the caller).
    let locator_buffer = largest_locator.checked_mul(2).ok_or_else(malformed)?;
    bytes.transient_bytes = bytes
        .transient_bytes
        .checked_add(u64::try_from(locator_buffer).map_err(|_| malformed())?)
        .ok_or_else(malformed)?;
    Ok(bytes)
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
    fn u32(&mut self) -> Result<usize, E> {
        Ok(u32::from_le_bytes([self.byte()?, self.byte()?, self.byte()?, self.byte()?]) as usize)
    }
}

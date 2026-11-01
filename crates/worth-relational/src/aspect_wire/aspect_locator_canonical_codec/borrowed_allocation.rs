//! Concrete locator allocation preflight over borrowed codec octets.
//! This does not construct or admit a locator.
use super::super::CanonicalDecodeStorage;
use super::tags;
use worth_foundational::facade::FieldKey;

pub(crate) fn decoded_bytes<E>(
    length: usize,
    next: impl FnMut() -> Result<u8, E>,
    malformed: impl Fn() -> E,
) -> Result<CanonicalDecodeStorage, E> {
    let mut input = Input {
        remaining: length,
        next,
        malformed,
    };
    let tag = input.byte()?;
    if !(tags::AUTHORITY_AUTHORITATIVE..=tags::AUTHORITY_RECEIPT_BEARING).contains(&input.byte()?) {
        return Err((input.malformed)());
    }
    let mut strings = input.string()?;
    let mut fields = 0;
    if tag == tags::STRUCT_FIELD {
        let count = input.u32()?;
        // Each field requires its u32 length before any field bytes.
        if count > input.remaining / 4 {
            return Err((input.malformed)());
        }
        fields = std::alloc::Layout::array::<FieldKey>(count)
            .map_err(|_| (input.malformed)())?
            .size();
        for _ in 0..count {
            strings = strings
                .checked_add(input.string()?)
                .ok_or_else(&input.malformed)?;
        }
    } else if tag != tags::WHOLE_ASPECT {
        return Err((input.malformed)());
    }
    if input.remaining != 0 {
        return Err((input.malformed)());
    }
    let retained = strings.checked_add(fields).ok_or_else(&input.malformed)?;
    // Encoded temporary key strings and the path-constructor input roster.
    Ok(CanonicalDecodeStorage {
        retained_bytes: u64::try_from(retained).map_err(|_| (input.malformed)())?,
        transient_bytes: u64::try_from(retained).map_err(|_| (input.malformed)())?,
    })
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
    fn string(&mut self) -> Result<usize, E> {
        let length = self.u32()?;
        if length > self.remaining {
            return Err((self.malformed)());
        }
        for _ in 0..length {
            self.byte()?;
        }
        // The reader first owns encoded String, then the concrete key owns its
        // validated copy. Both can coexist during the constructor call.
        Ok(length)
    }
}

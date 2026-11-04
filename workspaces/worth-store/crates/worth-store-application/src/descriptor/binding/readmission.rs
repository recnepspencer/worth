//! Decode recorded program meaning without granting installation authority.
use crate::descriptor::ApplicationHomeDescriptorDenial as Denial;
use worth_foundational::facade::{ApplicationHomeBinding, CanonicalDigestId, HomeProgramBinding};

pub(in crate::descriptor) fn decode(bytes: &[u8]) -> Result<ApplicationHomeBinding, Denial> {
    let mut read = RecordedBinding { remaining: bytes };
    let package = CanonicalDigestId::new(read.fixed()?);
    let schema = CanonicalDigestId::new(read.fixed()?);
    let (name, revision) = read.program()?;
    let selected = HomeProgramBinding::new(name.to_owned(), revision);
    let count = u32::from_le_bytes(read.fixed()?) as usize;
    // Each entry requires at least its length and digest. Bound before allocation.
    if count == 0 || count > read.remaining.len() / 36 {
        return Err(Denial::Malformed);
    }
    let mut roster = Vec::with_capacity(count);
    for _ in 0..count {
        let (name, revision) = read.program()?;
        let program = HomeProgramBinding::new(name.to_owned(), revision);
        if roster
            .last()
            .is_some_and(|previous: &HomeProgramBinding| previous.revision() >= program.revision())
        {
            return Err(Denial::Malformed);
        }
        roster.push(program);
    }
    if !read.remaining.is_empty() {
        return Err(Denial::Malformed);
    }
    ApplicationHomeBinding::new(package, schema, selected, roster).map_err(|_| Denial::Malformed)
}

/// Both returned binding boxes, the decoded roster capacity and its strings.
/// The requested binding moves from its existing owner; its backing is not cloned.
pub(in crate::descriptor) fn allocation_bytes(bytes: &[u8]) -> Result<u64, Denial> {
    let mut read = RecordedBinding { remaining: bytes };
    read.fixed::<32>()?;
    read.fixed::<32>()?;
    let (name, _) = read.program()?;
    let count = u32::from_le_bytes(read.fixed()?) as usize;
    if count == 0 || count > read.remaining.len() / 36 {
        return Err(Denial::Malformed);
    }
    let mut heap = std::mem::size_of::<ApplicationHomeBinding>()
        .checked_mul(2)
        .and_then(|heap| {
            count
                .checked_mul(std::mem::size_of::<HomeProgramBinding>())
                .and_then(|roster| heap.checked_add(roster))
        })
        .and_then(|heap| heap.checked_add(name.len()))
        .ok_or(Denial::SizeLimit)?;
    for _ in 0..count {
        let (name, _) = read.program()?;
        heap = heap.checked_add(name.len()).ok_or(Denial::SizeLimit)?;
    }
    if !read.remaining.is_empty() {
        return Err(Denial::Malformed);
    }
    u64::try_from(heap).map_err(|_| Denial::SizeLimit)
}

struct RecordedBinding<'a> {
    remaining: &'a [u8],
}
impl<'record> RecordedBinding<'record> {
    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Denial> {
        let (value, remaining) = self
            .remaining
            .split_first_chunk::<N>()
            .ok_or(Denial::Malformed)?;
        self.remaining = remaining;
        Ok(*value)
    }
    fn program(&mut self) -> Result<(&'record str, CanonicalDigestId), Denial> {
        let length = u32::from_le_bytes(self.fixed()?) as usize;
        let (name, remaining) = self
            .remaining
            .split_at_checked(length)
            .ok_or(Denial::Malformed)?;
        let name = std::str::from_utf8(name).map_err(|_| Denial::Malformed)?;
        self.remaining = remaining;
        Ok((name, CanonicalDigestId::new(self.fixed()?)))
    }
}
